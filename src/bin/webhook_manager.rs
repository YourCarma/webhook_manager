use webhook_manager::config::ServiceConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = ServiceConfig::new()?;
    let redis_config = config.storage();
    println!("{}", redis_config.host());
    Ok(())
}
