use anyhow::{Result, anyhow};

pub fn assert_env_var(name: &str) -> Result<String> {
    std::env::var(name).map_err(|e| anyhow!("missing environment variable: {name}"))
}
