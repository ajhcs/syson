// SPDX-License-Identifier: EPL-2.0
//! Explicit local selected-JSON storage candidate. No PostgreSQL/native schema claim.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};
use syson_document_reconcile::{Document as Observation, DocumentPlan, Uuid, plan_documents};
use syson_model::Document;
const LIMIT: usize = 8 * 1024 * 1024;
const DOCUMENT_LIMIT: usize = 1024 * 1024;
const BRIDGE: &str = include_str!("bridge.py");
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Io,
    Driver,
    Protocol,
    Limit,
    Serialization,
    InvalidDocument,
    ForeignObservation,
    Refused(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub project: Uuid,
    pub document: Uuid,
}
impl Key {
    fn wire(self) -> Value {
        let hex = |bytes: Uuid| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        json!([hex(self.project), hex(self.document)])
    }
}
/// Fields and provenance are private; observations belong to one Store handle.
pub struct Stored {
    token: Arc<()>,
    key: Key,
    name: String,
    content: String,
    document: Document,
    read_only: bool,
    revision: u64,
}
impl Stored {
    pub fn key(&self) -> Key {
        self.key
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
pub enum Change<'a> {
    Create {
        key: Key,
        name: &'a str,
        document: &'a Document,
        read_only: bool,
    },
    Update {
        before: &'a Stored,
        name: &'a str,
        document: &'a Document,
    },
    Delete {
        before: &'a Stored,
    },
}
pub struct Store {
    path: PathBuf,
    token: Arc<()>,
}
struct Limited {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("selected document byte budget"));
        }
        let required = self.bytes.len() + bytes.len();
        if required > self.bytes.capacity() {
            // Grow geometrically without allocating past this writer's budget.
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(required)
                .min(self.limit);
            self.bytes.reserve_exact(capacity - self.bytes.len());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn serialize(document: &Document, limit: usize) -> Result<String, Error> {
    if limit > DOCUMENT_LIMIT {
        return Err(Error::Limit);
    }
    let mut writer = Limited {
        bytes: vec![],
        limit,
    };
    serde_json::to_writer(&mut writer, document.value()).map_err(|_| Error::Serialization)?;
    String::from_utf8(writer.bytes).map_err(|_| Error::Serialization)
}
fn protocol_start(action: &str) -> Result<Limited, Error> {
    let mut input = Limited {
        bytes: vec![],
        limit: LIMIT,
    };
    input.write_all(b"{\"action\":").map_err(|_| Error::Limit)?;
    serde_json::to_writer(&mut input, action).map_err(|_| Error::Limit)?;
    input.write_all(b",\"value\":").map_err(|_| Error::Limit)?;
    Ok(input)
}
impl Store {
    /// The caller supplies a fresh owned absolute file path. Failure retains it.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(Error::Io);
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| Error::Io)?;
        let store = Self {
            path: path.into(),
            token: Arc::new(()),
        };
        store.call("initialize", Value::Null)?;
        Ok(store)
    }
    /// Never creates a missing DB; candidate format is checked before any write.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let store = Self {
            path: path.as_ref().into(),
            token: Arc::new(()),
        };
        store.call("load", json!(["", ""]))?;
        Ok(store)
    }
    fn call(&self, action: &str, value: Value) -> Result<Value, Error> {
        let mut input = protocol_start(action)?;
        serde_json::to_writer(&mut input, &value).map_err(|_| Error::Limit)?;
        input.write_all(b"}").map_err(|_| Error::Limit)?;
        self.invoke(input.bytes)
    }
    fn invoke(&self, input: Vec<u8>) -> Result<Value, Error> {
        if !self.path.is_absolute() {
            return Err(Error::Io);
        }
        if input.len() > LIMIT {
            return Err(Error::Limit);
        }
        let mut child = Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-B", "-c", BRIDGE])
            .arg(&self.path)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Error::Driver)?;
        let written = child.stdin.take().ok_or(Error::Driver)?.write_all(&input);
        if written.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Driver);
        }
        let output = child.wait_with_output().map_err(|_| Error::Driver)?;
        if !output.status.success() || output.stdout.len() > LIMIT {
            return Err(Error::Driver);
        }
        let result: Value = serde_json::from_slice(&output.stdout).map_err(|_| Error::Protocol)?;
        if let Some(error) = result["error"].as_str() {
            return Err(Error::Refused(error.into()));
        }
        Ok(result)
    }
    pub fn load(&self, key: Key) -> Result<Option<Stored>, Error> {
        let result = self.call("load", key.wire())?;
        if result.get("row") == Some(&Value::Null) {
            return Ok(None);
        }
        let row = result["row"]
            .as_array()
            .filter(|r| r.len() == 4)
            .ok_or(Error::Protocol)?;
        let (name, content) = (
            row[0].as_str().ok_or(Error::Protocol)?,
            row[1].as_str().ok_or(Error::Protocol)?,
        );
        let read_only = row[2].as_u64().filter(|v| *v <= 1).ok_or(Error::Protocol)? == 1;
        let revision = row[3]
            .as_u64()
            .filter(|v| *v > 0 && *v <= i64::MAX as u64)
            .ok_or(Error::Protocol)?;
        let document = Document::from_json(content).map_err(|_| Error::InvalidDocument)?;
        Ok(Some(Stored {
            token: self.token.clone(),
            key,
            name: name.into(),
            content: content.into(),
            document,
            read_only,
            revision,
        }))
    }
    /// Documents serialize before opening SQLite. A serializer failure cannot
    /// partially execute this batch; SQL refusals roll back BEGIN IMMEDIATE.
    pub fn transact(&self, changes: &[Change<'_>], document_bytes: usize) -> Result<(), Error> {
        if changes.is_empty() || changes.len() > 64 {
            return Err(Error::Limit);
        }
        let mut input = protocol_start("transact")?;
        input.write_all(b"[").map_err(|_| Error::Limit)?;
        for (index, change) in changes.iter().enumerate() {
            let (kind, key, name, content, revision, changed, read_only) = match change {
                Change::Create {
                    key,
                    name,
                    document,
                    read_only,
                } => (
                    "create",
                    *key,
                    *name,
                    serialize(document, document_bytes)?,
                    0,
                    true,
                    *read_only,
                ),
                Change::Update {
                    before,
                    name,
                    document,
                } => {
                    self.require(before)?;
                    let content = serialize(document, document_bytes)?;
                    let prior = Observation {
                        id: before.key.document,
                        name: before.name.clone(),
                        content: before.content.clone(),
                        is_read_only: before.read_only,
                        created_on: (),
                        last_modified_on: (),
                    };
                    let next = Observation {
                        id: before.key.document,
                        name: (*name).into(),
                        content: content.clone(),
                        is_read_only: before.read_only,
                        created_on: (),
                        last_modified_on: (),
                    };
                    let changed = matches!(
                        plan_documents(&[&prior], &[&next], &[]),
                        DocumentPlan::Replace { .. }
                    );
                    (
                        "update",
                        before.key,
                        *name,
                        content,
                        before.revision,
                        changed,
                        false,
                    )
                }
                Change::Delete { before } => {
                    self.require(before)?;
                    (
                        "delete",
                        before.key,
                        "",
                        String::new(),
                        before.revision,
                        false,
                        false,
                    )
                }
            };
            if name.len() > 4096 {
                return Err(Error::Limit);
            }
            if index > 0 {
                input.write_all(b",").map_err(|_| Error::Limit)?;
            }
            let mut operation = json!({"kind":kind,"key":key.wire(),"name":name,"revision":revision,"changed":changed,"read_only":read_only});
            // Move the one serialized document; do not clone or retain a batch.
            operation["content"] = Value::String(content);
            serde_json::to_writer(&mut input, &operation).map_err(|_| Error::Limit)?;
        }
        input.write_all(b"]}").map_err(|_| Error::Limit)?;
        if self.invoke(input.bytes)?["ok"] != true {
            return Err(Error::Protocol);
        }
        Ok(())
    }
    fn require(&self, before: &Stored) -> Result<(), Error> {
        if !Arc::ptr_eq(&self.token, &before.token) {
            return Err(Error::ForeignObservation);
        }
        Ok(())
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    #[test]
    fn writer_capacity_and_failed_append_respect_the_declared_budget() {
        let mut writer = Limited {
            bytes: vec![],
            limit: 17,
        };
        for chunk in [b"123".as_slice(), b"4567", b"8901234", b"567"] {
            writer.write_all(chunk).unwrap();
            assert!(writer.bytes.capacity() <= 17);
        }
        let before = writer.bytes.clone();
        assert!(writer.write_all(b"x").is_err());
        assert_eq!(writer.bytes, before);
    }

    #[test]
    fn json_escaping_and_the_final_delimiter_count_toward_the_byte_budget() {
        let value = json!({"name":"quote\" slash\\ nul\0 newline\n unicode🚗".repeat(30)});
        let expected = serde_json::to_vec(&json!([value.clone()])).unwrap();
        for budget in [expected.len() - 1, expected.len()] {
            let mut writer = Limited {
                bytes: vec![],
                limit: budget,
            };
            let result = writer
                .write_all(b"[")
                .and_then(|_| {
                    serde_json::to_writer(&mut writer, &value).map_err(std::io::Error::other)
                })
                .and_then(|_| writer.write_all(b"]"));
            assert_eq!(result.is_ok(), budget == expected.len());
            assert!(writer.bytes.len() <= budget && writer.bytes.capacity() <= budget);
            if result.is_ok() {
                assert_eq!(writer.bytes, expected);
            }
        }
    }
}
