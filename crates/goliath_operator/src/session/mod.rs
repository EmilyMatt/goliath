use crate::client::GoliathClient;
use crate::error::GoliathOperatorResult;
use crate::video::OperatorPipeline;
use goliath_common::{GoliathCommand, GoliathGstPipeline, MotorCommand, stop_main_loop};
use std::sync::Arc;

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
