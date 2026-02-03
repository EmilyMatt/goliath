use crate::GoliathVehicleResult;
use crate::session::GoliathVehicleSession;
use crate::video::capture_pipeline::ZedCamCaps;
use crate::video::encoding_pipeline::EncoderType;
use jetgpio::Gpio;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{MaybeTlsStream, accept_async};

pub(crate) struct GoliathServer {
    listener: TcpListener,
}

impl GoliathServer {
    pub(crate) async fn await_connection(
        &mut self,
        gpio: Arc<Gpio>,
    ) -> GoliathVehicleResult<GoliathVehicleSession> {
        let (new_connection, addr) = self.listener.accept().await?;
        // TODO: Move to TlsStream
        let ws_conn = accept_async(MaybeTlsStream::Plain(new_connection))
            .await
            .map_err(Box::new)?;
        GoliathVehicleSession::try_new(addr, ws_conn, ZedCamCaps::NOHD15, EncoderType::V4L2, gpio)
    }

    pub(crate) async fn try_new(port: usize) -> GoliathVehicleResult<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        Ok(Self { listener })
    }
}
