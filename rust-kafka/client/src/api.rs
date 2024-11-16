use anyhow::{anyhow, Result};
use gloo::net::http::RequestBuilder;

pub async fn get_channels() -> Result<Vec<String>> {
    // TODO config file with base url
    let url = "http://localhost:8001/channels";
    let response = RequestBuilder::new(url)
        .header("Accept", "application/json")
        .build()?
        .send()
        .await?;
    if !response.ok() {
        Err(anyhow!(
            "error response from {}, status({}, {})",
            url,
            response.status(),
            response.status_text()
        ))?;
    }
    Ok(response.json().await?)
}
