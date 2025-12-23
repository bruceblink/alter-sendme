use crate::core::common::canonicalized_path_to_string;
use crate::core::types::{
    AddrInfoOptions, AppHandle, SendArgs, SendOptions, SendResult, apply_options,
    get_or_create_secret, print_hash,
};
use anyhow::{Context, ensure};
use futures_buffered::BufferedStreamExt;
use indicatif::{HumanBytes, MultiProgress, ProgressBar};
use iroh::protocol::Router;
use iroh::{Endpoint, RelayMode, discovery::pkarr::PkarrPublisher};
use iroh_blobs::api::Store;
use iroh_blobs::api::blobs::{AddPathOptions, AddProgressItem, ImportMode};
use iroh_blobs::format::collection::Collection;
use iroh_blobs::{
    BlobFormat, BlobsProtocol, Hash,
    api::TempTag,
    protocol, provider,
    provider::events::{ConnectMode, EventMask, EventSender, RequestMode},
    store::fs::FsStore,
    ticket::BlobTicket,
};
use n0_future::StreamExt;
use n0_future::task::AbortOnDropHandle;
use rand::Rng;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use tokio::{select, sync::mpsc};
use walkdir::WalkDir;

fn emit_event(app_handle: &AppHandle, event_name: &str) {
    if let Some(e) = app_handle
        .as_ref()
        .and_then(|handle| handle.emit_event(event_name).err())
    {
        tracing::warn!("Failed to emit event {}: {}", event_name, e);
    }
}

fn emit_progress_event(
    app_handle: &AppHandle,
    bytes_transferred: u64,
    total_bytes: u64,
    speed_bps: f64,
) {
    if let Some(handle) = app_handle {
        let event_name = "transfer-progress";

        let speed_int = (speed_bps * 1000.0) as i64;

        let payload = format!("{}:{}:{}", bytes_transferred, total_bytes, speed_int);

        if let Err(e) = handle.emit_event_with_payload(event_name, &payload) {
            tracing::warn!("Failed to emit progress event: {}", e);
        }
    }
}

pub async fn start_share(
    path: PathBuf,
    options: SendOptions,
    app_handle: AppHandle,
) -> anyhow::Result<SendResult> {
    // 1️⃣ 构造 EventSender，从 AppHandle 发事件
    let (progress_tx, progress_rx) = mpsc::channel(32);
    let event_sender = Some(EventSender::new(
        progress_tx,
        EventMask {
            connected: ConnectMode::Notify,
            get: RequestMode::NotifyLog,
            ..EventMask::DEFAULT
        },
    ));

    // 2️⃣ 构造 send_core 参数
    let core_args = SendCoreArgs {
        path: path.clone(),
        options: options.clone(),
        event_sender,
    };

    // 3️⃣ 调用 send_core
    let SendCoreResult {
        router,
        temp_tag,
        store,
        blobs_data_dir,
        size,
        entry_type,
        hash,
    } = send_core(core_args).await?;

    // 4️⃣ 启动进度条 / 回调显示
    let progress_handle = n0_future::task::spawn(show_provide_progress_with_logging(
        progress_rx,
        app_handle,
        size,
        entry_type.clone(),
    ));

    // 5️⃣ 构造 ticket
    let mut addr = router.endpoint().addr();
    apply_options(&mut addr, options.ticket_type);
    let ticket = BlobTicket::new(addr, hash, BlobFormat::HashSeq);

    // 6️⃣ 返回 SendResult
    Ok(SendResult {
        ticket: ticket.to_string(),
        hash: hash.to_hex().to_string(),
        size,
        entry_type,
        router,
        temp_tag,
        blobs_data_dir,
        _progress_handle: AbortOnDropHandle::new(progress_handle),
        _store: store,
    })
}

async fn show_provide_progress_with_logging(
    mut recv: mpsc::Receiver<provider::events::ProviderMessage>,
    app_handle: AppHandle,
    total_file_size: u64,
    entry_type: String,
) -> anyhow::Result<()> {
    use n0_future::FuturesUnordered;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;

    let mut tasks = FuturesUnordered::new();

    #[derive(Clone)]
    struct TransferState {
        start_time: Instant,
        total_size: u64,
    }

    let transfer_states: Arc<Mutex<std::collections::HashMap<(u64, u64), TransferState>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    let active_requests = Arc::new(AtomicUsize::new(0));
    let completed_requests = Arc::new(AtomicUsize::new(0));
    let has_emitted_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let has_any_transfer = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let last_request_time: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

    loop {
        select! {
            biased;
            item = recv.recv() => {
                let Some(item) = item else {
                    break;
                };

                match item {
                    provider::events::ProviderMessage::ClientConnectedNotify(_msg) => {
                    }
                    provider::events::ProviderMessage::ConnectionClosed(_msg) => {
                    }
                    provider::events::ProviderMessage::GetRequestReceivedNotify(msg) => {
                        let connection_id = msg.connection_id;
                        let request_id = msg.request_id;

                        active_requests.fetch_add(1, Ordering::SeqCst);

                        let mut last_time = last_request_time.lock().await;
                        *last_time = Some(Instant::now());

                        let app_handle_task = app_handle.clone();
                        let transfer_states_task = transfer_states.clone();
                        let active_requests_task = active_requests.clone();
                        let completed_requests_task = completed_requests.clone();
                        let has_emitted_started_task = has_emitted_started.clone();
                        let has_any_transfer_task = has_any_transfer.clone();
                        let last_request_time_task = last_request_time.clone();
                        let entry_type_task = entry_type.clone();

                        let mut rx = msg.rx;
                        tasks.push(async move {
                            let mut transfer_started = false;
                            let mut request_completed = false;

                            while let Ok(Some(update)) = rx.recv().await {
                                match update {
                                    provider::events::RequestUpdate::Started(_m) => {
                                        if !transfer_started {
                                            transfer_states_task.lock().await.insert(
                                                (connection_id, request_id),
                                                TransferState {
                                                    start_time: Instant::now(),
                                                    total_size: total_file_size,
                                                }
                                            );

                                            if !has_emitted_started_task.swap(true, Ordering::SeqCst) {
                                                emit_event(&app_handle_task, "transfer-started");
                                            }

                                            transfer_started = true;
                                            has_any_transfer_task.store(true, Ordering::SeqCst);
                                        }
                                    }
                                    provider::events::RequestUpdate::Progress(m) => {
                                        if !transfer_started {
                                            if !has_emitted_started_task.swap(true, Ordering::SeqCst) {
                                                emit_event(&app_handle_task, "transfer-started");
                                            }
                                            transfer_started = true;
                                            has_any_transfer_task.store(true, Ordering::SeqCst);
                                        }

                                        if let Some(state) = transfer_states_task.lock().await.get(&(connection_id, request_id)) {
                                            let elapsed = state.start_time.elapsed().as_secs_f64();
                                            let speed_bps = if elapsed > 0.0 {
                                                m.end_offset as f64 / elapsed
                                            } else {
                                                0.0
                                            };

                                            emit_progress_event(&app_handle_task, m.end_offset, state.total_size, speed_bps);
                                        }
                                    }
                                    provider::events::RequestUpdate::Completed(_m) => {
                                        if transfer_started && !request_completed {
                                            transfer_states_task.lock().await.remove(&(connection_id, request_id));
                                            request_completed = true;

                                            let completed = completed_requests_task.fetch_add(1, Ordering::SeqCst) + 1;
                                            let active = active_requests_task.load(Ordering::SeqCst);

                                            // For directories, require at least 2 completed requests
                                            // to avoid false completion from metadata transfer
                                            let min_required = if entry_type_task == "directory" { 2 } else { 1 };

                                            if completed >= active
                                                && completed >= min_required
                                                && has_any_transfer_task.load(Ordering::SeqCst) {
                                                let active_before_wait = active;

                                                tokio::time::sleep(Duration::from_millis(500)).await;

                                                let completed_after = completed_requests_task.load(Ordering::SeqCst);
                                                let active_after = active_requests_task.load(Ordering::SeqCst);

                                                let new_requests_arrived = active_after > active_before_wait;

                                                let has_active_transfers = {
                                                    let states = transfer_states_task.lock().await;
                                                    !states.is_empty()
                                                };

                                                let last_request_recent = {
                                                    let last_time = last_request_time_task.lock().await;
                                                    if let Some(time) = *last_time {
                                                        time.elapsed() < Duration::from_millis(500)
                                                    } else {
                                                        false
                                                    }
                                                };

                                                if completed_after >= active_after
                                                    && completed_after >= min_required
                                                    && !new_requests_arrived
                                                    && !has_active_transfers
                                                    && !last_request_recent {
                                                    emit_event(&app_handle_task, "transfer-completed");
                                                }
                                            }
                                        }
                                    }
                                    provider::events::RequestUpdate::Aborted(_m) => {
                                        tracing::warn!("Request aborted: conn {} req {}",
                                            connection_id, request_id);
                                        if transfer_started && !request_completed {
                                            transfer_states_task.lock().await.remove(&(connection_id, request_id));
                                            request_completed = true;

                                            let completed = completed_requests_task.fetch_add(1, Ordering::SeqCst) + 1;
                                            let active = active_requests_task.load(Ordering::SeqCst);

                                            if completed >= active {
                                                emit_event(&app_handle_task, "transfer-failed");
                                            }
                                        }
                                    }
                                }
                            }

                            if transfer_started && !request_completed {
                                let completed = completed_requests_task.fetch_add(1, Ordering::SeqCst) + 1;
                                let active = active_requests_task.load(Ordering::SeqCst);

                                // For directories, require at least 2 completed requests
                                // to avoid false completion from metadata transfer
                                let min_required = if entry_type_task == "directory" { 2 } else { 1 };

                                if completed >= active
                                    && completed >= min_required
                                    && has_any_transfer_task.load(Ordering::SeqCst) {
                                    let active_before_wait = active;

                                    tokio::time::sleep(Duration::from_millis(500)).await;

                                    let completed_after = completed_requests_task.load(Ordering::SeqCst);
                                    let active_after = active_requests_task.load(Ordering::SeqCst);

                                    let new_requests_arrived = active_after > active_before_wait;

                                    let has_active_transfers = {
                                        let states = transfer_states_task.lock().await;
                                        !states.is_empty()
                                    };

                                    let last_request_recent = {
                                        let last_time = last_request_time_task.lock().await;
                                        if let Some(time) = *last_time {
                                            time.elapsed() < Duration::from_millis(500)
                                        } else {
                                            false
                                        }
                                    };

                                    if completed_after >= active_after
                                        && completed_after >= min_required
                                        && !new_requests_arrived
                                        && !has_active_transfers
                                        && !last_request_recent {
                                        emit_event(&app_handle_task, "transfer-completed");
                                    }
                                }
                            }
                        });
                    }
                    _ => {
                    }
                }
            }
            Some(_) = tasks.next(), if !tasks.is_empty() => {
            }
        }
    }

    while tasks.next().await.is_some() {}

    if has_any_transfer.load(Ordering::SeqCst) {
        let completed = completed_requests.load(Ordering::SeqCst);
        let active = active_requests.load(Ordering::SeqCst);

        // For directories, require at least 2 completed requests
        // to avoid false completion from metadata transfer
        let min_required = if entry_type == "directory" { 2 } else { 1 };

        if completed >= active && completed >= min_required && completed > 0 {
            emit_event(&app_handle, "transfer-completed");
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct SendCoreArgs {
    pub path: PathBuf,
    pub options: SendOptions,

    /// iroh-blobs 的事件发送器（可选）
    /// send_core 不关心事件内容，只负责接入
    pub event_sender: Option<EventSender>,
}

pub struct SendCoreResult {
    pub router: Router,
    pub temp_tag: TempTag,
    pub store: FsStore,
    pub blobs_data_dir: PathBuf,
    pub size: u64,
    pub entry_type: String,
    pub hash: Hash,
}

pub async fn send_core(args: SendCoreArgs) -> anyhow::Result<SendCoreResult> {
    let SendCoreArgs {
        path,
        options,
        event_sender,
    } = args;

    // --------------------------------------------------
    // endpoint / relay 配置
    // --------------------------------------------------
    let secret_key = get_or_create_secret()?;
    let relay_mode: RelayMode = options.relay_mode.clone().into();

    let mut builder = Endpoint::builder()
        .alpns(vec![protocol::ALPN.to_vec()])
        .secret_key(secret_key)
        .relay_mode(relay_mode.clone());

    if options.ticket_type == AddrInfoOptions::Id {
        builder = builder.discovery(PkarrPublisher::n0_dns());
    }
    if let Some(addr) = options.magic_ipv4_addr {
        builder = builder.bind_addr_v4(addr);
    }
    if let Some(addr) = options.magic_ipv6_addr {
        builder = builder.bind_addr_v6(addr);
    }

    // --------------------------------------------------
    // blobs 数据目录
    // --------------------------------------------------
    let suffix = rand::rng().random::<[u8; 16]>();
    let temp_base = std::env::temp_dir();
    let blobs_data_dir = temp_base.join(format!(".sendmer-send-{}", hex::encode(suffix)));

    if blobs_data_dir.exists() {
        anyhow::bail!(
            "can not share twice from the same directory: {}",
            blobs_data_dir.display()
        );
    }

    let cwd = std::env::current_dir()?;
    if cwd.join(&path) == cwd {
        anyhow::bail!("can not share from the current directory");
    }

    tokio::fs::create_dir_all(&blobs_data_dir).await?;

    // --------------------------------------------------
    // endpoint / store / blobs
    // --------------------------------------------------
    let endpoint = builder.bind().await?;

    let store = FsStore::load(&blobs_data_dir).await?;

    let blobs = BlobsProtocol::new(&store, event_sender);

    // --------------------------------------------------
    // import
    // --------------------------------------------------
    let (temp_tag, size, _collection) = import(path.clone(), blobs.store())
        .await
        .context("failed to import path")?;

    let hash: Hash = temp_tag.hash();

    let entry_type = if path.is_file() {
        "file".to_string()
    } else {
        "directory".to_string()
    };

    // --------------------------------------------------
    // router
    // --------------------------------------------------
    let router = Router::builder(endpoint)
        .accept(protocol::ALPN, blobs.clone())
        .spawn();

    // 等待 endpoint 上线（relay）
    let ep = router.endpoint();
    timeout(Duration::from_secs(30), async {
        if !matches!(relay_mode, RelayMode::Disabled) {
            let _ = ep.online().await;
        }
    })
    .await
    .context("timeout waiting for endpoint to become online")?;

    // --------------------------------------------------
    // 返回运行态
    // --------------------------------------------------
    Ok(SendCoreResult {
        router,
        temp_tag,
        store,
        blobs_data_dir,
        size,
        entry_type,
        hash,
    })
}

pub async fn send(args: SendArgs) -> anyhow::Result<()> {
    // 1️⃣ 构造 EventSender，用于 CLI 进度显示
    let (progress_tx, progress_rx) = mpsc::channel(32);
    let mp = MultiProgress::new();
    let mp2 = mp.clone();
    let progress_handle = AbortOnDropHandle::new(n0_future::task::spawn(
        crate::core::common::show_provide_progress(mp2, progress_rx),
    ));

    let event_sender = Some(EventSender::new(
        progress_tx,
        EventMask {
            connected: ConnectMode::Notify,
            get: RequestMode::NotifyLog,
            ..EventMask::DEFAULT
        },
    ));

    // 2️⃣ 构造 send_core 参数
    let core_args = SendCoreArgs {
        path: args.path.clone(),
        options: SendOptions {
            relay_mode: args.common.relay,
            ticket_type: args.ticket_type,
            magic_ipv4_addr: args.common.magic_ipv4_addr,
            magic_ipv6_addr: args.common.magic_ipv6_addr,
        },
        event_sender,
    };

    // 3️⃣ 调用 send_core
    let SendCoreResult {
        router,
        temp_tag,
        store: _store,
        blobs_data_dir,
        size,
        entry_type,
        hash,
    } = send_core(core_args).await?;

    // 4️⃣ 构造 ticket
    let mut addr = router.endpoint().addr();
    apply_options(&mut addr, args.ticket_type);
    let ticket = BlobTicket::new(addr, hash, BlobFormat::HashSeq);

    // 5️⃣ CLI 输出
    println!(
        "imported {} {}, {}, hash {}",
        entry_type,
        args.path.display(),
        HumanBytes(size),
        print_hash(&hash, args.common.format),
    );

    println!("to get this data, use");
    println!("sendmer receive {ticket}");

    // 6️⃣ 等待 ctrl-c
    tokio::signal::ctrl_c().await?;
    drop(temp_tag);

    println!("shutting down");
    tokio::time::timeout(Duration::from_secs(2), router.shutdown()).await??;

    // 删除临时目录
    tokio::fs::remove_dir_all(blobs_data_dir).await?;
    drop(router);

    // 等待进度条完成
    progress_handle.await.ok();

    Ok(())
}

/// Import from a file or directory into the database.
///
/// The returned tag always refers to a collection. If the input is a file, this
/// is a collection with a single blob, named like the file.
///
/// If the input is a directory, the collection contains all the files in the
/// directory.
/// 带进度条 / GUI 的 import
pub async fn import(path: PathBuf, db: &Store) -> anyhow::Result<(TempTag, u64, Collection)> {
    // 调用核心 import_core
    let (temp_tag, size, collection) = import_core(path.clone(), db).await?;

    // 如果需要显示进度条，可以在这里模拟一个总进度（可选）
    let op = ProgressBar::new(size);
    op.set_message(format!("Imported {} files ({})", collection.len(), size));
    op.finish_and_clear();

    Ok((temp_tag, size, collection))
}

pub async fn import_core(path: PathBuf, db: &Store) -> anyhow::Result<(TempTag, u64, Collection)> {
    let parallelism = num_cpus::get();
    let path = path.canonicalize()?;
    ensure!(path.exists(), "path {} does not exist", path.display());

    let root = path.parent().context("failed to get parent of path")?;

    // 遍历文件，将目录结构扁平化为 (name, path) 列表
    let files = WalkDir::new(path.clone()).into_iter();
    let data_sources: Vec<(String, PathBuf)> = files
        .map(|entry| {
            let entry = entry?;
            if !entry.file_type().is_file() {
                return Ok(None);
            }
            let path = entry.into_path();
            let relative = path.strip_prefix(root)?;
            let name = canonicalized_path_to_string(relative, true)?;
            Ok(Some((name, path)))
        })
        .filter_map(anyhow::Result::transpose)
        .collect::<anyhow::Result<Vec<_>>>()?;

    // 异步导入文件，使用 num_cpus 并行
    let mut names_and_tags = futures::stream::iter(data_sources)
        .map(|(name, path)| {
            let db = db.clone();
            async move {
                let import = db.add_path_with_opts(AddPathOptions {
                    path,
                    mode: ImportMode::TryReference,
                    format: BlobFormat::Raw,
                });
                let mut stream = import.stream().await;
                let mut item_size = 0;
                let temp_tag = loop {
                    let item = stream
                        .next()
                        .await
                        .context("import stream ended without a tag")?;
                    match item {
                        AddProgressItem::Size(size) => {
                            item_size = size;
                        }
                        AddProgressItem::CopyProgress(_) => {}
                        AddProgressItem::CopyDone => {}
                        AddProgressItem::OutboardProgress(_) => {}
                        AddProgressItem::Error(cause) => {
                            anyhow::bail!("error importing {}: {}", name, cause);
                        }
                        AddProgressItem::Done(tt) => {
                            break tt;
                        }
                    }
                };
                anyhow::Ok((name, temp_tag, item_size))
            }
        })
        .buffered_unordered(parallelism)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    // 按文件名排序
    names_and_tags.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));

    // 总大小
    let size = names_and_tags.iter().map(|(_, _, s)| *s).sum::<u64>();

    // 构造 collection，并保留 temp tags 防止被 gc
    let (collection, tags) = names_and_tags
        .into_iter()
        .map(|(name, tag, _)| ((name, tag.hash()), tag))
        .unzip::<_, _, Collection, Vec<_>>();

    // store collection，得到总的 temp_tag
    let temp_tag = collection.clone().store(db).await?;

    // 释放 temp_tag，数据由 collection 保持
    drop(tags);
    Ok((temp_tag, size, collection))
}
