#![feature(async_await)]

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::try_init()?;

    Ok(())
}
