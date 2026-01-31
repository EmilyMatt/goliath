#![allow(clippy::upper_case_acronyms)]
#![deny(clippy::clone_on_ref_ptr)]

use crate::image_proc::{convert_image_to_screen_space, load_goliath_logo, resize_image};
use crate::server::GoliathServer;
use crate::ssd1306::create_ssd_connection;
use error::GoliathVehicleResult;
use goliath_common::{initiate_gstreamer, start_main_loop};

mod error;
mod image_proc;
mod motors;
mod server;
mod session;
mod ssd1306;
mod video;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> GoliathVehicleResult<()> {
    goliath_common::common_init_for_trace()?;
    initiate_gstreamer()?;

    let mut ssd = create_ssd_connection()?;

    let main_logo = load_goliath_logo().and_then(|img| {
        convert_image_to_screen_space(
            resize_image(img, ssd.width(), ssd.height()),
            ssd.width(),
            ssd.height(),
        )
    })?;

    ssd.update_screen(0, &main_logo)?;

    let mut operator_connection = GoliathServer::try_new(5000).await?;
    loop {
        log::info!("Awaiting new connection");
        let mut session_ctx = operator_connection.await_connection().await?;

        tokio::spawn(async move { session_ctx.run().await }).await??;

        start_main_loop();
        log::info!("Main loop closed");
    }
}
