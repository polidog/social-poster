use super::{base, call, Error};
use serde::Deserialize;
use serde_json::json;
use std::time::SystemTime;

#[derive(Deserialize)]
pub struct Bluesky {
    #[serde(default = "default_host")]
    host: String,
    identifier: String,
    password: String,
}

fn default_host() -> String {
    "bsky.social".into()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    did: String,
    access_jwt: String,
}

#[derive(Deserialize)]
struct Record {
    uri: String,
}

impl Bluesky {
    pub fn post(&self, text: &str) -> Result<String, Error> {
        let base = base(&self.host);
        let sess: Session = call(
            &format!("{base}/xrpc/com.atproto.server.createSession"),
            None,
            json!({"identifier": self.identifier, "password": self.password}),
        )?;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        // ponytail: no facets, so links/mentions are posted as plain text. Add rich text when needed.
        let rec: Record = call(
            &format!("{base}/xrpc/com.atproto.repo.createRecord"),
            Some(&sess.access_jwt),
            json!({"repo": sess.did, "collection": "app.bsky.feed.post", "record": {
                "$type": "app.bsky.feed.post", "text": text, "createdAt": rfc3339(now as i64)}}),
        )?;
        // at://did/app.bsky.feed.post/<rkey>
        let rkey = rec.uri.rsplit('/').next().unwrap_or_default();
        Ok(format!("https://bsky.app/profile/{}/post/{rkey}", sess.did))
    }
}

fn rfc3339(secs: i64) -> String {
    let (days, t) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // civil_from_days by Howard Hinnant
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", t / 3600, t % 3600 / 60, t % 60)
}

#[test]
fn rfc3339_works() {
    assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
    assert_eq!(rfc3339(951782400), "2000-02-29T00:00:00Z");
    assert_eq!(rfc3339(1790812799), "2026-09-30T23:59:59Z");
}
