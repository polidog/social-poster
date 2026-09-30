use super::{base, call, Error};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Misskey {
    host: String,
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Created {
    created_note: Note,
}

#[derive(Deserialize)]
struct Note {
    id: String,
}

impl Misskey {
    pub fn post(&self, text: &str) -> Result<String, Error> {
        let base = base(&self.host);
        // Token in the body ("i") works on older Misskey versions too.
        let res: Created = call(&format!("{base}/api/notes/create"), None, json!({"i": self.token, "text": text}))?;
        Ok(format!("{base}/notes/{}", res.created_note.id))
    }
}
