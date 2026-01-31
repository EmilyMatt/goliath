use crate::GoliathVehicleResult;
use crate::error::GoliathVehicleError;
use crate::motors::MotorsContoller;
use crate::video::capture_pipeline::{CapturePipeline, ZedCamCaps};
use crate::video::encoding_pipeline::{EncoderType, EncodingPipline};
use crate::video::rtp_pipeline::RTPPipeline;
use futures_util::stream::StreamExt;
pub use goliath_common::{GoliathCommand, MotorCommand};
use goliath_common::{GoliathGstPipeline, stop_main_loop};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub(crate) struct GoliathVehicleSession {
    operator_ws: WebSocketStream<MaybeTlsStream<TcpStream>>,

    capture_pipeline: Arc<CapturePipeline>,

    motors_cmd_tx: mpsc::Sender<MotorCommand>,
    motors_thread: Option<thread::JoinHandle<GoliathVehicleResult<()>>>,
}

impl GoliathVehicleSession {
    pub(crate) fn try_new(
        operator_addr: SocketAddr,
        operator_ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        capture_caps: ZedCamCaps,
        encoder_type: EncoderType,
    ) -> GoliathVehicleResult<Self> {
        let ip = match operator_addr {
            SocketAddr::V4(ip) => ip.to_string(),
            SocketAddr::V6(_) => {
                return Err(GoliathVehicleError::GeneralError(
                    "IPV6 is not supported".to_string(),
                ));
            }
        };
        let rtp_pipeline = Arc::new(RTPPipeline::try_new(vec![(ip, 8000)])?);
        let encoding_pipeline = Arc::new(EncodingPipline::try_new(encoder_type, rtp_pipeline)?);
        let capture_pipeline = Arc::new(CapturePipeline::try_new(capture_caps, encoding_pipeline)?);

        let (motors_cmd_tx, motors_cmd_rx) = mpsc::channel::<MotorCommand>(32);
        let motors_thread = thread::Builder::new()
            .name("MotorsThread".to_string())
            .spawn({
                let mut motors = MotorsContoller::try_new()?;
                move || motors.run_thread(motors_cmd_rx)
            })?;

        Ok(Self {
            operator_ws,

            capture_pipeline,

            motors_cmd_tx,
            motors_thread: Some(motors_thread),
        })
    }

    async fn handle_command(&mut self, cmd: GoliathCommand) -> GoliathVehicleResult<()> {
        match cmd {
            GoliathCommand::Motor(motor_cmd) => {
                self.motors_cmd_tx
                    .send(motor_cmd)
                    .await
                    .map_err(|err| GoliathVehicleError::TokioSendError(err.to_string()))?;
            }
        }
        Ok(())
    }

    pub(crate) async fn run(&mut self) -> GoliathVehicleResult<()> {
        log::info!("Starting Session");
        self.capture_pipeline.start_pipeline(None)?;

        while let Some(msg) = self.operator_ws.next().await {
            match msg {
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(Message::Close(frame)) => {
                    if let Some(frame) = frame {
                        log::info!("Got close frame from websocket: {frame:?}");
                    } else {
                        log::info!("Got close message from websocket with unknown reason");
                    }
                    break;
                }
                Ok(Message::Text(text)) => {
                    log::warn!("Got unexpected text: {text}")
                }
                Ok(Message::Binary(bytes)) => match GoliathCommand::read_from_bytes(&bytes) {
                    Ok(cmd) => {
                        if let Err(err) = self.handle_command(cmd).await {
                            log::error!("Failed to handle command: {err}");
                            break;
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to read command: {err} (command: {:?})", bytes);
                    }
                },
                Ok(Message::Frame(_)) => {
                    log::debug!("Got raw frame message, ignoring");
                }
                Err(err) => {
                    log::error!("Received error from websocket: {err:?}");
                    break;
                }
            }
        }

        // Client disconnected, stop everything
        self.motors_cmd_tx
            .send(MotorCommand::Thrust(0.0))
            .await
            .map_err(|err| GoliathVehicleError::TokioSendError(err.to_string()))?;
        self.motors_cmd_tx
            .send(MotorCommand::Steer(0.0))
            .await
            .map_err(|err| GoliathVehicleError::TokioSendError(err.to_string()))?;
        self.motors_cmd_tx
            .send(MotorCommand::End)
            .await
            .map_err(|err| GoliathVehicleError::TokioSendError(err.to_string()))?;

        self.capture_pipeline.stop_pipeline().ok();
        if let Some(motors_thread) = self.motors_thread.take() {
            motors_thread.join().ok();
        }

        stop_main_loop();
        Ok(())
    }
}
