mod error;

use crate::error::GoliathServerResult;
use goliath_common::common_init_for_trace;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> GoliathServerResult<()> {
    common_init_for_trace()?;

    Ok(())
}
