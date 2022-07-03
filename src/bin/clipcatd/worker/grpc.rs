use std::sync::Arc;

use chrono::{DateTime, Local};
use snafu::ResultExt;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use clipcat::{
    grpc::{self, ManagerService, MonitorService},
    ClipboardManager, ClipboardMonitor,
};

use crate::error::{self, Error};
pub enum WatchEventType {
    ClipboardChanged,
    ClipboardUpdated,
    NotifyEvent,
    WatchEvent,
}

pub struct WatchEvent {
    ev_type: WatchEventType,
    clip_id: usize,
    timestamp: DateTime<Local>
}

pub enum Message {
    Shutdown,
    WatchEvent,
}

#[allow(clippy::never_loop)]
pub fn start(
    grpc_addr: std::net::SocketAddr,
    clipboard_monitor: Arc<Mutex<ClipboardMonitor>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
) -> (mpsc::UnboundedSender<Message>, JoinHandle<Result<(), Error>>) {
    let server = {
        let monitor_service = MonitorService::new(clipboard_monitor);
        let manager_service = ManagerService::new(clipboard_manager);

        tonic::transport::Server::builder()
            .add_service(grpc::MonitorServer::new(monitor_service))
            .add_service(grpc::ManagerServer::new(manager_service))
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let join_handle = tokio::spawn(async move {
        tracing::info!("gRPC service listening on {}", grpc_addr);

        server
            .serve_with_shutdown(grpc_addr, async move {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Message::Shutdown => {
                            tracing::info!("gRPC service is shutting down gracefully");
                            return;
                        }
                        Message::WatchEvent => {
                            tracing::info!("gRPC service had watch event");
                            return;
                        }
                    }
                }
            })
            .await
            .context(error::ServeGrpc)?;

        Ok(())
    });
    (tx, join_handle)
}
