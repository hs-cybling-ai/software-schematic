use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const EMBEDDING_SCHEMA: &str = "ssw.embedding/v1";
pub const CHUNKER_VERSION: &str = "ssw.markdown-chunks/v1";
pub const HEADER_START: &str = "<!-- software-schematic-embedding\n";
pub const HEADER_END: &str = "\n-->\n\n";
pub const MAX_HEADER_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CHUNKS: usize = 4096;
pub const MAX_DIMENSIONS: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmbeddingEnvelope {
    pub schema: String,
    pub owner: String,
    pub body_hash: String,
    pub chunker: String,
    pub model: String,
    pub dimensions: usize,
    pub chunks: Vec<EmbeddingChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmbeddingChunk {
    pub id: String,
    pub content_hash: String,
    pub ordinal: usize,
    pub heading_path: Vec<String>,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct ParsedMarkdown {
    pub body: String,
    pub envelope: Option<EmbeddingEnvelope>,
    pub diagnostic: Option<String>,
}

pub fn body_hash(body: &str) -> String {
    format!("sha256:{}", hex(&Sha256::digest(body.as_bytes())))
}

pub fn embedding_revision(envelopes: &[&EmbeddingEnvelope]) -> String {
    let mut values = envelopes
        .iter()
        .map(|value| serde_json::to_vec(value).unwrap_or_default())
        .collect::<Vec<_>>();
    values.sort();
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value);
    }
    format!("sha256:{}", hex(&hasher.finalize()))
}

pub fn parse_markdown(physical: &str) -> ParsedMarkdown {
    if !physical.starts_with(HEADER_START) {
        return ParsedMarkdown {
            body: physical.to_owned(),
            envelope: None,
            diagnostic: None,
        };
    }
    let payload_start = HEADER_START.len();
    let Some(relative_end) = physical[payload_start..].find(HEADER_END) else {
        return ParsedMarkdown {
            body: String::new(),
            envelope: None,
            diagnostic: Some("unterminated software-schematic embedding header".into()),
        };
    };
    let payload_end = payload_start + relative_end;
    let body_start = payload_end + HEADER_END.len();
    let body = physical[body_start..].to_owned();
    if payload_end > MAX_HEADER_BYTES {
        return ParsedMarkdown {
            body,
            envelope: None,
            diagnostic: Some("software-schematic embedding header exceeds size limit".into()),
        };
    }
    match serde_json::from_str::<EmbeddingEnvelope>(&physical[payload_start..payload_end]) {
        Ok(envelope) => ParsedMarkdown {
            body,
            envelope: Some(envelope),
            diagnostic: None,
        },
        Err(error) => ParsedMarkdown {
            body,
            envelope: None,
            diagnostic: Some(format!(
                "invalid software-schematic embedding header: {error}"
            )),
        },
    }
}

pub fn serialize_markdown(envelope: &EmbeddingEnvelope, body: &str) -> Result<String> {
    validate_envelope_shape(envelope)?;
    let payload = serde_json::to_string(envelope)
        .map_err(|error| Error::Message(format!("serialize embedding header: {error}")))?;
    if payload.len() > MAX_HEADER_BYTES {
        return Err(Error::Message("embedding header exceeds size limit".into()));
    }
    Ok(format!("{HEADER_START}{payload}{HEADER_END}{body}"))
}

pub fn validate_envelope_shape(envelope: &EmbeddingEnvelope) -> Result<()> {
    if envelope.schema != EMBEDDING_SCHEMA {
        return Err(Error::Message(format!(
            "unsupported embedding schema {}",
            envelope.schema
        )));
    }
    if envelope.owner.is_empty() || envelope.owner.len() > 4096 {
        return Err(Error::Message("invalid embedding owner".into()));
    }
    if envelope.body_hash.len() != 71 || !envelope.body_hash.starts_with("sha256:") {
        return Err(Error::Message("invalid embedding body hash".into()));
    }
    if envelope.chunker != CHUNKER_VERSION {
        return Err(Error::Message("unsupported embedding chunker".into()));
    }
    if envelope.model.is_empty() || envelope.model.len() > 512 {
        return Err(Error::Message("invalid embedding model identity".into()));
    }
    if envelope.dimensions == 0 || envelope.dimensions > MAX_DIMENSIONS {
        return Err(Error::Message("invalid embedding dimensions".into()));
    }
    if envelope.chunks.len() > MAX_CHUNKS {
        return Err(Error::Message("embedding chunk limit exceeded".into()));
    }
    for (ordinal, chunk) in envelope.chunks.iter().enumerate() {
        if chunk.ordinal != ordinal
            || chunk.id.is_empty()
            || chunk.content_hash.is_empty()
            || chunk.vector.len() != envelope.dimensions
            || chunk.vector.iter().any(|value| !value.is_finite())
        {
            return Err(Error::Message(format!(
                "invalid embedding chunk at ordinal {ordinal}"
            )));
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> EmbeddingEnvelope {
        EmbeddingEnvelope {
            schema: EMBEDDING_SCHEMA.into(),
            owner: "main.cmmn#diagram".into(),
            body_hash: body_hash("# Café\r\n"),
            chunker: CHUNKER_VERSION.into(),
            model: "test".into(),
            dimensions: 2,
            chunks: vec![EmbeddingChunk {
                id: "chunk-0".into(),
                content_hash: "hash".into(),
                ordinal: 0,
                heading_path: vec!["Café".into()],
                vector: vec![0.25, -0.5],
            }],
        }
    }

    #[test]
    fn deterministic_round_trip_preserves_body_bytes() {
        let value = envelope();
        let physical = serialize_markdown(&value, "# Café\r\n").unwrap();
        assert_eq!(serialize_markdown(&value, "# Café\r\n").unwrap(), physical);
        let parsed = parse_markdown(&physical);
        assert_eq!(parsed.body, "# Café\r\n");
        assert_eq!(parsed.envelope, Some(value));
    }

    #[test]
    fn ordinary_front_matter_is_authored_body() {
        let body = "---\ntitle: Example\n---\n# Body\n";
        assert_eq!(parse_markdown(body).body, body);
    }

    #[test]
    fn malformed_payload_is_hidden_but_body_is_recovered() {
        let physical = format!("{HEADER_START}not-json{HEADER_END}# Body\n");
        let parsed = parse_markdown(&physical);
        assert_eq!(parsed.body, "# Body\n");
        assert!(parsed.envelope.is_none());
        assert!(parsed.diagnostic.is_some());
    }

    #[test]
    fn rejects_non_finite_and_oversized_vectors() {
        let mut invalid = envelope();
        invalid.chunks[0].vector[0] = f32::NAN;
        assert!(validate_envelope_shape(&invalid).is_err());
        invalid.chunks[0].vector = vec![0.0; MAX_DIMENSIONS + 1];
        invalid.dimensions = MAX_DIMENSIONS + 1;
        assert!(validate_envelope_shape(&invalid).is_err());
    }

    #[test]
    fn json_arrays_are_portable_and_bounded() {
        let encoded = serde_json::to_vec(&envelope()).unwrap();
        assert!(encoded.starts_with(b"{"));
        assert!(encoded.len() < 1024);
    }
}
