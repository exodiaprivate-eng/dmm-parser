// 1.0.8: SkillInfo has new BuffData variants (tags >120).
// Blob mode for safe roundtrip until all new variants are decoded.

crate::pabgh_blob_table! {
    pub struct SkillInfoBlob<'a> {
        key: u32,
        blob_field: body,
    }
}

impl<'a> SkillInfoBlob<'a> {
    pub fn to_json_dict(&self) -> serde_json::Map<String, serde_json::Value> {
        use base64::Engine;
        let mut m = serde_json::Map::new();
        m.insert("key".into(), serde_json::Value::from(self.key));
        m.insert("string_key".into(), serde_json::Value::from(
            std::str::from_utf8(self.string_key.data.as_bytes()).unwrap_or("")));
        m.insert("is_blocked".into(), serde_json::Value::from(self.is_blocked));
        m.insert("_body_b64".into(), serde_json::Value::from(
            base64::engine::general_purpose::STANDARD.encode(&self.body)));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &serde_json::Map<String, serde_json::Value>) -> std::io::Result<()> {
        use crate::binary::BinaryWrite;
        use base64::Engine;
        let key = obj.get("key").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        key.write_to(w)?;
        let sk = obj.get("string_key").and_then(|v| v.as_str()).unwrap_or("");
        (sk.len() as u32).write_to(w)?;
        w.extend_from_slice(sk.as_bytes());
        let blocked = obj.get("is_blocked").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        blocked.write_to(w)?;
        if let Some(b64) = obj.get("_body_b64").and_then(|v| v.as_str()) {
            let body = base64::engine::general_purpose::STANDARD.decode(b64)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            w.extend_from_slice(&body);
        }
        Ok(())
    }
}
