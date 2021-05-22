use anyhow::Result;

use pueue_daemon_lib::run;

#[async_std::main]
async fn main() -> Result<()> {
    run().await
}
