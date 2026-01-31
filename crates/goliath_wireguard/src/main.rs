use crate::error::GoliathWireguardResult;
use goliath_common::common_init_for_trace;

mod error;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> GoliathWireguardResult<()> {
    common_init_for_trace()?;

    Ok(())
}
