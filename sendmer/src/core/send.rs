use crate::core::types::{SendResult, SendOptions, AddrInfoOptions, apply_options, get_or_create_secret, AppHandle};
use anyhow::Context;
use iroh::{
    discovery::pkarr::PkarrPublisher,
    Endpoint, RelayMode,
};
use iroh_blobs::{api::{
    TempTag,
}, protocol, provider::{
    events::{ConnectMode, EventMask, EventSender, RequestMode},
}, store::fs::FsStore, ticket::BlobTicket, BlobFormat, BlobsProtocol, Hash};
use n0_future::{task::AbortOnDropHandle};
use rand::Rng;
use std::{
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};
use iroh::protocol::Router;
use tokio::{select, sync::mpsc};
use n0_future::StreamExt;
use tokio::time::timeout;

fn emit_event(app_handle: &AppHandle, event_name: &str) {
    if let Some(e) = app_handle
        .as_ref()
        .and_then(|handle| handle.emit_event(event_name).err()) {
        tracing::warn!("Failed to emit event {}: {}", event_name, e);
    }
}

fn emit_progress_event(app_handle: &AppHandle, bytes_transferred: u64, total_bytes: u64, speed_bps: f64) {
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

pub fn canonicalized_path_to_string(
    path: impl AsRef<Path>,
    must_be_relative: bool,
) -> anyhow::Result<String> {
    let mut path_str = String::new();
    let parts = path
        .as_ref()
        .components()
        .filter_map(|c| match c {
            Component::Normal(x) => {
                let c = match x.to_str() {
                    Some(c) => c,
                    None => return Some(Err(anyhow::anyhow!("invalid character in path"))),
                };

                if !c.contains('/') && !c.contains('\\') {
                    Some(Ok(c))
                } else {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                }
            }
            Component::RootDir => {
                if must_be_relative {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                } else {
                    path_str.push('/');
                    None
                }
            }
            _ => Some(Err(anyhow::anyhow!("invalid path component {:?}", c))),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let parts = parts.join("/");
    path_str.push_str(&parts);
    Ok(path_str)
}

async fn show_provide_progress_with_logging(
    mut recv: mpsc::Receiver<iroh_blobs::provider::events::ProviderMessage>,
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
                    iroh_blobs::provider::events::ProviderMessage::ClientConnectedNotify(_msg) => {
                    }
                    iroh_blobs::provider::events::ProviderMessage::ConnectionClosed(_msg) => {
                    }
                    iroh_blobs::provider::events::ProviderMessage::GetRequestReceivedNotify(msg) => {
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
                                    iroh_blobs::provider::events::RequestUpdate::Started(_m) => {
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
                                    iroh_blobs::provider::events::RequestUpdate::Progress(m) => {
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
                                    iroh_blobs::provider::events::RequestUpdate::Completed(_m) => {
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
                                    iroh_blobs::provider::events::RequestUpdate::Aborted(_m) => {
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
    
    while tasks.next().await.is_some() {
    }
    
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
    let blobs_data_dir =
        temp_base.join(format!(".sendmer-send-{}", hex::encode(suffix)));

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

    let blobs = BlobsProtocol::new(
        &store,
        event_sender,
    );

    // --------------------------------------------------
    // import
    // --------------------------------------------------
    let (temp_tag, size, _collection) =
        crate::core::main_reference::import(path.clone(), blobs.store())
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
    }).await.context("timeout waiting for endpoint to become online")?;

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
