use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use socketioxide::{SocketIo, adapter::Adapter, extract::SocketRef};
use tracing::{info, warn};

use crate::domain::UnknownRequest;

pub const UNKNOWN_REQUEST_CAPTURED_EVENT: &str = "unknown_request:captured";

#[async_trait::async_trait]
pub trait RealtimeNotifier: Send + Sync {
    async fn unknown_request_captured(&self, request: UnknownRequest);
}

#[derive(Clone)]
pub struct SocketIoNotifier<A: Adapter> {
    io: SocketIo<A>,
}

impl<A: Adapter> SocketIoNotifier<A> {
    pub fn new(io: SocketIo<A>) -> Self {
        Self { io }
    }
}

#[async_trait::async_trait]
impl<A: Adapter> RealtimeNotifier for SocketIoNotifier<A> {
    async fn unknown_request_captured(&self, request: UnknownRequest) {
        let payload = UnknownRequestEvent::from(request);
        if let Err(error) = self.io.emit(UNKNOWN_REQUEST_CAPTURED_EVENT, &payload).await {
            warn!(%error, "failed to emit unknown request realtime event");
        }
    }
}

pub fn register_admin_namespace<A>(
    io: &SocketIo<A>,
) -> <A as socketioxide_core::adapter::CoreAdapter<socketioxide::adapter::Emitter>>::InitRes
where
    A: Adapter + socketioxide_core::adapter::DefinedAdapter,
{
    io.ns("/", async |socket: SocketRef<A>| {
        info!(sid = %socket.id, "admin Socket.IO client connected");
    })
}

#[derive(Debug, Serialize)]
struct UnknownRequestEvent {
    #[serde(flatten)]
    request: UnknownRequest,
    body_base64: Option<String>,
    body_text: Option<String>,
}

impl From<UnknownRequest> for UnknownRequestEvent {
    fn from(request: UnknownRequest) -> Self {
        let body_base64 = request.body.as_ref().map(|body| STANDARD.encode(body));
        let body_text = request
            .body
            .as_ref()
            .and_then(|body| String::from_utf8(body.clone()).ok());

        Self {
            request,
            body_base64,
            body_text,
        }
    }
}
