use crate::client::GoliathClient;
use crate::error::GoliathOperatorResult;
use crate::video::OperatorPipeline;
use goliath_common::{GoliathCommand, GoliathGstPipeline, MotorCommand, stop_main_loop};
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub(crate) struct GoliathOperatorSession {
    client_conn: GoliathClient,
    operator_pipeline: Arc<OperatorPipeline>,
}

impl GoliathOperatorSession {
    pub(crate) fn try_new(client_conn: GoliathClient) -> GoliathOperatorResult<Self> {
        let operator_pipeline = Arc::new(OperatorPipeline::try_new(
            gstreamer::Caps::builder("application/x-rtp")
                .field("encoding-name", "H264")
                .build(),
        )?);
        Ok(Self {
            client_conn,
            operator_pipeline,
        })
    }

    pub(crate) async fn run(&mut self) -> GoliathOperatorResult<()> {
        log::info!("Starting Session");
        self.operator_pipeline.start_pipeline(None)?;

        let controller_socket = UdpSocket::bind("0.0.0.0:6000").await?;
        let mut mtu_buffer = [0u8; 1400];

        // Main loop
        loop {
            match self.client_conn.poll_report() {
                Ok(Some(report)) => {
                    log::info!("Received report: {report:?}");
                }
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error while polling reports: {err}");
                    break;
                }
            }

            controller_socket.readable().await?;

            match controller_socket.try_recv(&mut mtu_buffer) {
                Ok(read) => {
                    let read_slice = &mtu_buffer[..read];

                    #[derive(Debug, serde::Serialize, serde::Deserialize)]
                    struct ControllerInfo {
                        thrust: f32,
                        steer: f32,
                    }

                    if let Ok(msg) = serde_json::from_slice::<ControllerInfo>(read_slice) {
                        log::info!("Got msg: {msg:?}");
                        if let Err(e) = self
                            .client_conn
                            .send_command(GoliathCommand::Motor(MotorCommand::Thrust(msg.thrust)))
                            .await
                        {
                            log::error!("Error while sending command: {e}");
                            break;
                        }

                        if let Err(e) = self
                            .client_conn
                            .send_command(GoliathCommand::Motor(MotorCommand::Steer(msg.steer)))
                            .await
                        {
                            log::error!("Error while sending command: {e}");
                            break;
                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => {
                    log::error!("Error while reading UDP packet: {e}");
                    break;
                }
            };
        }

        self.client_conn
            .send_command(GoliathCommand::Motor(MotorCommand::Thrust(0.0)))
            .await
            .ok();
        self.client_conn
            .send_command(GoliathCommand::Motor(MotorCommand::Steer(0.0)))
            .await
            .ok();

        self.operator_pipeline.stop_pipeline().ok();
        stop_main_loop();
        Ok(())
    }
}
