use crate::GoliathVehicleResult;
use crate::session::GoliathSession;
use crate::video::capture_pipeline::ZedCamCaps;
use crate::video::encoding_pipeline::EncoderType;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub(crate) struct GoliathServer {
    listener: TcpListener,
}

impl GoliathServer {
    pub(crate) async fn await_connection(&mut self) -> GoliathVehicleResult<GoliathSession> {
        let (new_connection, addr) = self.listener.accept().await?;
        let ws_conn = accept_async(new_connection).await.map_err(Box::new)?;
        GoliathSession::try_new(addr, ws_conn, ZedCamCaps::NOHD15, EncoderType::V4L2)
    }

    pub(crate) async fn try_new(port: usize) -> GoliathVehicleResult<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        Ok(Self { listener })
    }
}
