mod client;
mod error;
mod session;
mod video;

use crate::client::GoliathClient;
use crate::error::GoliathOperatorResult;
use crate::session::GoliathOperatorSession;
use goliath_common::{common_init_for_trace, initiate_gstreamer, start_main_loop};
use std::net::Ipv4Addr;
use tokio::runtime::Handle;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> GoliathOperatorResult<()> {
    common_init_for_trace()?;
    initiate_gstreamer()?;

    loop {
        log::info!("Attempting new connection");
        // TODO: Replace this with clap arg, then with a wireguard-provided address
        let client_ws = GoliathClient::try_new(Ipv4Addr::new(192, 168, 0, 100), 5000).await?;
        log::info!("Connected. creating session");
        let mut session_ctx = GoliathOperatorSession::try_new(client_ws)?;

        Handle::current().spawn_blocking(start_main_loop);
        tokio::spawn(async move { session_ctx.run().await }).await??;
    }
}
