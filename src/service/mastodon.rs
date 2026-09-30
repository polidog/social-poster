use super::{base, call, Error};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Mastodon {
    host: String,
    token: String,
}

#[derive(Deserialize)]
struct Status {
    url: String,
}

impl Mastodon {
    pub fn post(&self, text: &str) -> Result<String, Error> {
        let url = format!("{}/api/v1/statuses", base(&self.host));
        let res: Status = call(&url, Some(&self.token), json!({"status": text}))?;
        Ok(res.url)
    }
}
