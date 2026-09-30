use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod bluesky;
mod mastodon;
mod misskey;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Http(#[from] ureq::Error),
    #[error("API returned {status}: {body}")]
    Api { status: u16, body: String },
    #[error("unexpected response: {0}")]
    Decode(#[from] serde_json::Error),
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Service {
    Misskey(misskey::Misskey),
    Mastodon(mastodon::Mastodon),
    Bluesky(bluesky::Bluesky),
}

impl Service {
    /// Posts `text` and returns the URL of the created post.
    pub fn post(&self, text: &str) -> Result<String, Error> {
        match self {
            Service::Misskey(s) => s.post(text),
            Service::Mastodon(s) => s.post(text),
            Service::Bluesky(s) => s.post(text),
        }
    }
}

fn call<T: DeserializeOwned>(url: &str, bearer: Option<&str>, body: impl Serialize) -> Result<T, Error> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut req = agent.post(url);
    if let Some(t) = bearer {
        req = req.header("Authorization", &format!("Bearer {t}"));
    }
    let mut res = req.send_json(body)?;
    let status = res.status();
    let text = res.body_mut().read_to_string()?;
    if !status.is_success() {
        return Err(Error::Api { status: status.as_u16(), body: text });
    }
    Ok(serde_json::from_str(&text)?)
}

/// "misskey.io" → https://misskey.io; "http://localhost:8080" is kept as is (for local testing).
fn base(host: &str) -> String {
    if host.contains("://") { host.to_string() } else { format!("https://{host}") }
}
