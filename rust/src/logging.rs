use anyhow::Result;

pub fn init(_level: Option<String>) -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    Ok(())
}
