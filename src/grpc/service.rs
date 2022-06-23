use std::sync::Arc;

use tokio::sync::Mutex;

use tonic::{Request, Response, Status};

use crate::{
    grpc::protobuf::{
        manager_server::Manager, monitor_server::Monitor, BatchRemoveRequest, BatchRemoveResponse,
        ClearRequest, ClearResponse, DisableMonitorRequest, EnableMonitorRequest,
        GetCurrentClipboardRequest, GetCurrentClipboardResponse, GetCurrentPrimaryRequest,
        GetCurrentPrimaryResponse, GetMonitorStateRequest, GetRequest, GetResponse,
        InfoRequest, InfoResponse, InsertRequest,
        InsertResponse, LengthRequest, LengthResponse, ListRequest, ListResponse,
        MarkAsClipboardRequest, MarkAsClipboardResponse, MarkAsPrimaryRequest,
        MarkAsPrimaryResponse, MonitorStateReply, RemoveRequest, RemoveResponse,
        ToggleMonitorRequest, UpdateRequest, UpdateResponse,
    },
    ClipboardManager, ClipboardMonitor,
};

pub struct ManagerService {
    manager: Arc<Mutex<ClipboardManager>>,
}

impl ManagerService {
    pub fn new(manager: Arc<Mutex<ClipboardManager>>) -> ManagerService {
        ManagerService { manager }
    }
}

#[tonic::async_trait]
impl Manager for ManagerService {
    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        let InsertRequest { data, clipboard_type } = request.into_inner();
        let clipboard_type = clipboard_type.into();
        let id = {
            let mut manager = self.manager.lock().await;
            let id = manager.insert(crate::ClipboardData::new(&data, clipboard_type));

            if clipboard_type == crate::ClipboardType::Clipboard {
                let _ = manager.mark_as_clipboard(id).await;
            }

            id
        };
        Ok(Response::new(InsertResponse { id }))
    }

    async fn remove(
        &self,
        request: Request<RemoveRequest>,
    ) -> Result<Response<RemoveResponse>, Status> {
        let id = request.into_inner().id;
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.remove(id)
        };
        Ok(Response::new(RemoveResponse { ok }))
    }

    async fn batch_remove(
        &self,
        request: Request<BatchRemoveRequest>,
    ) -> Result<Response<BatchRemoveResponse>, Status> {
        let ids = request.into_inner().ids;
        let ids = {
            let mut manager = self.manager.lock().await;
            ids.into_iter().filter(|id| manager.remove(*id)).collect()
        };
        Ok(Response::new(BatchRemoveResponse { ids }))
    }

    async fn clear(
        &self,
        _request: Request<ClearRequest>,
    ) -> Result<Response<ClearResponse>, Status> {
        {
            let mut manager = self.manager.lock().await;
            manager.clear();
        }
        Ok(Response::new(ClearResponse {}))
    }

    async fn info(&self, request: Request<InfoRequest>) -> Result<Response<InfoResponse>, Status> {
        let InfoRequest { id } = request.into_inner();
        let data = {
            let manager = self.manager.lock().await;
            manager.get(id).map(Into::into)
        };
        Ok(Response::new(InfoResponse { data }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let GetRequest { id } = request.into_inner();
        let data = {
            let manager = self.manager.lock().await;
            manager.get(id).map(Into::into)
        };
        Ok(Response::new(GetResponse { data }))
    }

    async fn get_current_clipboard(
        &self,
        _request: Request<GetCurrentClipboardRequest>,
    ) -> Result<Response<GetCurrentClipboardResponse>, Status> {
        let data = {
            let manager = self.manager.lock().await;
            manager.get_current_clipboard().map(|clip| clip.clone().into())
        };
        Ok(Response::new(GetCurrentClipboardResponse { data }))
    }

    async fn get_current_primary(
        &self,
        _request: Request<GetCurrentPrimaryRequest>,
    ) -> Result<Response<GetCurrentPrimaryResponse>, Status> {
        let data = {
            let manager = self.manager.lock().await;
            manager.get_current_primary().map(|clip| clip.clone().into())
        };
        Ok(Response::new(GetCurrentPrimaryResponse { data }))
    }

    async fn list(&self, _request: Request<ListRequest>) -> Result<Response<ListResponse>, Status> {
        let data = {
            let manager = self.manager.lock().await;
            manager.list().into_iter().map(Into::into).collect()
        };
        Ok(Response::new(ListResponse { data }))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let UpdateRequest { id, data } = request.into_inner();
        let (ok, new_id) = {
            let mut manager = self.manager.lock().await;
            manager.replace(id, &data)
        };
        Ok(Response::new(UpdateResponse { ok, new_id }))
    }

    async fn mark_as_clipboard(
        &self,
        request: Request<MarkAsClipboardRequest>,
    ) -> Result<Response<MarkAsClipboardResponse>, Status> {
        let MarkAsClipboardRequest { id } = request.into_inner();
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.mark_as_clipboard(id).await.is_ok()
        };
        Ok(Response::new(MarkAsClipboardResponse { ok }))
    }

    async fn mark_as_primary(
        &self,
        request: Request<MarkAsPrimaryRequest>,
    ) -> Result<Response<MarkAsPrimaryResponse>, Status> {
        let MarkAsPrimaryRequest { id } = request.into_inner();
        let ok = {
            let mut manager = self.manager.lock().await;
            manager.mark_as_primary(id).await.is_ok()
        };
        Ok(Response::new(MarkAsPrimaryResponse { ok }))
    }

    async fn length(
        &self,
        _request: Request<LengthRequest>,
    ) -> Result<Response<LengthResponse>, Status> {
        let length = {
            let manager = self.manager.lock().await;
            manager.len() as u64
        };
        Ok(Response::new(LengthResponse { length }))
    }
}

pub struct MonitorService {
    monitor: Arc<Mutex<ClipboardMonitor>>,
}

impl MonitorService {
    #[inline]
    pub fn new(monitor: Arc<Mutex<ClipboardMonitor>>) -> MonitorService {
        MonitorService { monitor }
    }
}

#[tonic::async_trait]
impl Monitor for MonitorService {
    async fn enable_monitor(
        &self,
        _request: Request<EnableMonitorRequest>,
    ) -> Result<Response<MonitorStateReply>, Status> {
        let state = {
            let mut monitor = self.monitor.lock().await;
            monitor.enable();
            MonitorStateReply { state: monitor.state().into() }
        };
        dbg!(&state);

        Ok(Response::new(state))
    }

    async fn disable_monitor(
        &self,
        _request: Request<DisableMonitorRequest>,
    ) -> Result<Response<MonitorStateReply>, Status> {
        let state = {
            let mut monitor = self.monitor.lock().await;
            monitor.disable();
            MonitorStateReply { state: monitor.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn toggle_monitor(
        &self,
        _request: Request<ToggleMonitorRequest>,
    ) -> Result<Response<MonitorStateReply>, Status> {
        let state = {
            let mut monitor = self.monitor.lock().await;
            monitor.toggle();
            MonitorStateReply { state: monitor.state().into() }
        };

        Ok(Response::new(state))
    }

    async fn get_monitor_state(
        &self,
        _request: Request<GetMonitorStateRequest>,
    ) -> Result<Response<MonitorStateReply>, Status> {
        let state = {
            let monitor = self.monitor.lock().await;
            MonitorStateReply { state: monitor.state().into() }
        };

        Ok(Response::new(state))
    }
}
