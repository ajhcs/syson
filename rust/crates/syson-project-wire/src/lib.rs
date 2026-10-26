// SPDX-License-Identifier: EPL-2.0
//! Lossless payload-token transport for the local selected prepared-project API.
//! Not a native SysON archive or proof that decoded inputs are prepared/valid.
#![forbid(unsafe_code)]
use serde::{
    Deserialize, Serialize,
    de::{SeqAccess, Visitor},
};
use serde_json::value::RawValue;
use std::{fmt, io::Write, marker::PhantomData};
use syson_project_preparation::{
    PreparedProject, ProjectInput, RepresentationInput, SemanticInput, SemanticVersions,
};

pub const FORMAT: &str = "syson-project-wire/1";
pub const MAX_ITEMS: usize = 64;
pub const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
pub const MAX_TOTAL_PAYLOAD_BYTES: usize = 8 * MAX_PAYLOAD_BYTES;
pub const MAX_METADATA_BYTES: usize = 1024;
pub const MAX_ENVELOPE_BYTES: usize = 10 * MAX_PAYLOAD_BYTES;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Limit(&'static str),
    UnsupportedFormat,
    Json(String),
    PayloadPadding,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(scope) => write!(f, "project wire limit exceeded: {scope}"),
            Self::UnsupportedFormat => f.write_str("unsupported local project wire format"),
            Self::Json(detail) => write!(f, "invalid project wire JSON: {detail}"),
            Self::PayloadPadding => f.write_str(
                "prepared payload has outside-value whitespace; cannot export losslessly",
            ),
        }
    }
}
impl std::error::Error for Error {}
// A non-transparent newtype is required: serde treats a missing bare Option as
// None, but a missing newtype field is an error. Present null remains explicit.
#[derive(Serialize, Deserialize)]
struct Stamp(Option<String>);
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Versions {
    connector: Stamp,
    annotation: Stamp,
    view: Stamp,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Semantic<'a> {
    #[serde(borrow)]
    json: &'a RawValue,
    versions: Versions,
    references_version: Stamp,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Representation<'a> {
    kind: String,
    #[serde(borrow)]
    json: &'a RawValue,
    control_version: Stamp,
    diagram_version: Stamp,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<'a> {
    format: String,
    #[serde(borrow, deserialize_with = "limited")]
    documents: Vec<Semantic<'a>>,
    #[serde(borrow, deserialize_with = "limited")]
    representations: Vec<Representation<'a>>,
}
fn limited<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Rows<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Rows<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("bounded item array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
            let mut out = Vec::new();
            while let Some(value) = a.next_element()? {
                if out.len() == MAX_ITEMS {
                    return Err(serde::de::Error::custom("item limit"));
                }
                out.push(value);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Rows(PhantomData))
}
fn metadata(value: &Option<String>) -> Result<(), Error> {
    if value.as_ref().is_some_and(|s| s.len() > MAX_METADATA_BYTES) {
        return Err(Error::Limit("metadata bytes"));
    }
    Ok(())
}
fn counts(documents: usize, representations: usize) -> Result<(), Error> {
    if documents.saturating_add(representations) > MAX_ITEMS {
        return Err(Error::Limit("items"));
    }
    Ok(())
}
fn payload(value: &str, total: &mut usize) -> Result<(), Error> {
    if value.len() > MAX_PAYLOAD_BYTES {
        return Err(Error::Limit("payload bytes"));
    }
    *total = total
        .checked_add(value.len())
        .filter(|n| *n <= MAX_TOTAL_PAYLOAD_BYTES)
        .ok_or(Error::Limit("aggregate payload bytes"))?;
    Ok(())
}
fn raw(value: &str) -> Result<&RawValue, Error> {
    let parsed: &RawValue = serde_json::from_str(value).map_err(|e| Error::Json(e.to_string()))?;
    // Prepared wire is a complete compact value. Do not silently drop outside-
    // value whitespace if a future preparation implementation changes that.
    if parsed.get() != value {
        return Err(Error::PayloadPadding);
    }
    Ok(parsed)
}
fn validate(e: &Envelope<'_>) -> Result<(), Error> {
    if e.format != FORMAT {
        return Err(Error::UnsupportedFormat);
    }
    counts(e.documents.len(), e.representations.len())?;
    let mut total = 0;
    for d in &e.documents {
        payload(d.json.get(), &mut total)?;
        for s in [
            &d.versions.connector,
            &d.versions.annotation,
            &d.versions.view,
            &d.references_version,
        ] {
            metadata(&s.0)?;
        }
    }
    for r in &e.representations {
        payload(r.json.get(), &mut total)?;
        if r.kind.len() > MAX_METADATA_BYTES {
            return Err(Error::Limit("kind bytes"));
        }
        metadata(&r.control_version.0)?;
        metadata(&r.diagram_version.0)?;
    }
    Ok(())
}
struct Output {
    bytes: Vec<u8>,
    limited: bool,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > MAX_ENVELOPE_BYTES {
            self.limited = true;
            return Err(std::io::Error::other("envelope byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Export only the authoritative wire snapshot from a completely prepared
/// project. Never serialize its typed B01 Document to reconstruct payloads.
/// Payload bytes are embedded as RawValue, not quoted/reformatted JSON strings.
/// All six independent stamp families are included, with explicit null fields.
/// Private bounded buffering exposes no partial bytes on any error.
/// Unprepared inputs cannot be exported through this API:
/// ```compile_fail
/// let input = syson_project_preparation::ProjectInput::default();
/// syson_project_wire::encode(&input).unwrap();
/// ```
pub fn encode(prepared: &PreparedProject) -> Result<Vec<u8>, Error> {
    let wire = prepared.wire();
    counts(wire.documents.len(), wire.representations.len())?;
    let mut total = 0;
    for d in &wire.documents {
        payload(&d.json, &mut total)?;
        for s in [
            &d.versions.connector,
            &d.versions.annotation,
            &d.versions.view,
            &d.references_version,
        ] {
            metadata(s)?;
        }
    }
    for r in &wire.representations {
        payload(&r.json, &mut total)?;
        if r.kind.len() > MAX_METADATA_BYTES {
            return Err(Error::Limit("kind bytes"));
        }
        metadata(&r.control_version)?;
        metadata(&r.diagram_version)?;
    }
    let documents = wire
        .documents
        .iter()
        .map(|d| {
            Ok(Semantic {
                json: raw(&d.json)?,
                versions: Versions {
                    connector: Stamp(d.versions.connector.clone()),
                    annotation: Stamp(d.versions.annotation.clone()),
                    view: Stamp(d.versions.view.clone()),
                },
                references_version: Stamp(d.references_version.clone()),
            })
        })
        .collect::<Result<_, Error>>()?;
    let representations = wire
        .representations
        .iter()
        .map(|r| {
            Ok(Representation {
                kind: r.kind.clone(),
                json: raw(&r.json)?,
                control_version: Stamp(r.control_version.clone()),
                diagram_version: Stamp(r.diagram_version.clone()),
            })
        })
        .collect::<Result<_, Error>>()?;
    let envelope = Envelope {
        format: FORMAT.into(),
        documents,
        representations,
    };
    let mut out = Output {
        bytes: Vec::new(),
        limited: false,
    };
    if let Err(e) = serde_json::to_writer(&mut out, &envelope) {
        return Err(if out.limited {
            Error::Limit("envelope bytes")
        } else {
            Error::Json(e.to_string())
        });
    }
    Ok(out.bytes)
}
/// Decode this strict local envelope into untrusted C03b input, not a prepared
/// project. The real preparation API must interpret the transported stamps and
/// revalidate model/representation admission before publication.
/// RawValue preserves each complete JSON value-token byte range, including its
/// interior whitespace, member order, duplicate payload members, integers and
/// Unicode. Whitespace separating an envelope key/value is not payload data.
/// Duplicate/unknown envelope and metadata fields refuse; payloads stay opaque.
pub fn decode(bytes: &[u8]) -> Result<ProjectInput, Error> {
    if bytes.len() > MAX_ENVELOPE_BYTES {
        return Err(Error::Limit("envelope bytes"));
    }
    let e: Envelope<'_> = serde_json::from_slice(bytes).map_err(|e| Error::Json(e.to_string()))?;
    validate(&e)?;
    Ok(ProjectInput {
        documents: e
            .documents
            .into_iter()
            .map(|d| SemanticInput {
                json: d.json.get().into(),
                versions: SemanticVersions {
                    connector: d.versions.connector.0,
                    annotation: d.versions.annotation.0,
                    view: d.versions.view.0,
                },
                references_version: d.references_version.0,
            })
            .collect(),
        representations: e
            .representations
            .into_iter()
            .map(|r| RepresentationInput {
                kind: r.kind,
                json: r.json.get().into(),
                control_version: r.control_version.0,
                diagram_version: r.diagram_version.0,
            })
            .collect(),
    })
}
