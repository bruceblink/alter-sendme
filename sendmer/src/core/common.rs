//! Command line arguments.

use std::{
    collections::BTreeMap,
    path::{Component, Path},
    sync::{Arc, Mutex},
    time::{Duration},
};

use anyhow::{Result};
use console::style;
use indicatif::{
    MultiProgress, ProgressBar, ProgressStyle,
};
use iroh_blobs::{provider::{
    events::{ProviderMessage, RequestUpdate},
}};
use n0_future::{FuturesUnordered, StreamExt};
use tokio::select;
use tokio::sync::mpsc;
use tracing::error;


/// This function converts an already canonicalized path to a string.
///
/// If `must_be_relative` is true, the function will fail if any component of the path is
/// `Component::RootDir`
///
/// This function will also fail if the path is non canonical, i.e. contains
/// `..` or `.`, or if the path components contain any windows or unix path
/// separators.
pub fn canonicalized_path_to_string(
    path: impl AsRef<Path>,
    must_be_relative: bool,
) -> Result<String> {
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
        .collect::<Result<Vec<_>>>()?;
    let parts = parts.join("/");
    path_str.push_str(&parts);
    Ok(path_str)
}

#[derive(Debug)]
struct PerConnectionProgress {
    node_id: String,
    requests: BTreeMap<u64, ProgressBar>,
}

async fn per_request_progress(
    mp: MultiProgress,
    connection_id: u64,
    request_id: u64,
    connections: Arc<Mutex<BTreeMap<u64, PerConnectionProgress>>>,
    mut rx: irpc::channel::mpsc::Receiver<RequestUpdate>,
) {
    let pb = mp.add(ProgressBar::hidden());
    let node_id = if let Some(connection) = connections.lock().unwrap().get_mut(&connection_id) {
        connection.requests.insert(request_id, pb.clone());
        connection.node_id.clone()
    } else {
        error!("got request for unknown connection {connection_id}");
        return;
    };
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}",
        ).unwrap()
        .progress_chars("#>-"),
    );
    while let Ok(Some(msg)) = rx.recv().await {
        match msg {
            RequestUpdate::Started(msg) => {
                pb.set_message(format!(
                    "n {} r {}/{} i {} # {}",
                    node_id,
                    connection_id,
                    request_id,
                    msg.index,
                    msg.hash.fmt_short()
                ));
                pb.set_length(msg.size);
            }
            RequestUpdate::Progress(msg) => {
                pb.set_position(msg.end_offset);
            }
            RequestUpdate::Completed(_) => {
                if let Some(msg) = connections.lock().unwrap().get_mut(&connection_id) {
                    msg.requests.remove(&request_id);
                };
            }
            RequestUpdate::Aborted(_) => {
                if let Some(msg) = connections.lock().unwrap().get_mut(&connection_id) {
                    msg.requests.remove(&request_id);
                };
            }
        }
    }
    pb.finish_and_clear();
    mp.remove(&pb);
}

pub async fn show_provide_progress(
    mp: MultiProgress,
    mut recv: mpsc::Receiver<ProviderMessage>,
) -> Result<()> {
    let connections = Arc::new(Mutex::new(BTreeMap::new()));
    let mut tasks = FuturesUnordered::new();
    loop {
        select! {
            biased;
            item = recv.recv() => {
                let Some(item) = item else {
                    break;
                };

                match item {
                    ProviderMessage::ClientConnectedNotify(msg) => {
                        let node_id = msg.endpoint_id.map(|id| id.fmt_short().to_string()).unwrap_or_else(|| "?".to_string());
                        let connection_id = msg.connection_id;
                        connections.lock().unwrap().insert(
                            connection_id,
                            PerConnectionProgress {
                                requests: BTreeMap::new(),
                                node_id,
                            },
                        );
                    }
                    ProviderMessage::ConnectionClosed(msg) => {
                        if let Some(connection) = connections.lock().unwrap().remove(&msg.connection_id) {
                            for pb in connection.requests.values() {
                                pb.finish_and_clear();
                                mp.remove(pb);
                            }
                        }
                    }
                    ProviderMessage::GetRequestReceivedNotify(msg) => {
                        let request_id = msg.request_id;
                        let connection_id = msg.connection_id;
                        let connections = connections.clone();
                        let mp = mp.clone();
                        tasks.push(per_request_progress(mp, connection_id, request_id, connections, msg.rx));
                    }
                    _ => {}
                }
            }
            Some(_) = tasks.next(), if !tasks.is_empty() => {}
        }
    }
    while tasks.next().await.is_some() {}
    Ok(())
}

const TICK_MS: u64 = 250;

fn make_download_progress() -> ProgressBar {
    let pb = ProgressBar::hidden();
    pb.enable_steady_tick(Duration::from_millis(TICK_MS));
    pb.set_style(
        ProgressStyle::with_template("{prefix}{spinner:.green}{msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_prefix(format!("{} ", style("[3/4]").bold().dim()));
    pb.set_message("Downloading ...".to_string());
    pb
}

pub async fn show_download_progress(
    mp: MultiProgress,
    mut recv: mpsc::Receiver<u64>,
    local_size: u64,
    total_size: u64,
) -> Result<()> {
    let op = mp.add(make_download_progress());
    op.set_length(total_size);
    while let Some(offset) = recv.recv().await {
        op.set_position(local_size + offset);
    }
    op.finish_and_clear();
    Ok(())
}