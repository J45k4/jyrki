use crate::llm::{GenRequest, SuccessfullGenResponse};
use anyhow::bail;
use reqwest::{Client, StatusCode};

pub async fn gen(req: GenRequest, client: Client) -> anyhow::Result<SuccessfullGenResponse> {
    let body = serde_json::to_string(&req)?;
    let apikey = std::env::var("OPENAI_API_KEY")?;

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", apikey))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    let status_code = res.status();
    let text = match status_code {
        StatusCode::OK => res.text().await?,
        _ => {
            log::error!("request failed with code {}", status_code);
            bail!("request failed with code {}", status_code)
        }
    };
    let res: SuccessfullGenResponse = serde_json::from_str(&text)?;
    Ok(res)
}
