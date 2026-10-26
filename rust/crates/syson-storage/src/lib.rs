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
        if !self.path.is_absolute() {
            return Err(Error::Io);
        }
        let input = serde_json::to_vec(&json!({"action":action,"value":value}))
            .map_err(|_| Error::Serialization)?;
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
        let mut operations = Vec::new();
        for change in changes {
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
            operations.push(json!({"kind":kind,"key":key.wire(),"name":name,"content":content,"revision":revision,"changed":changed,"read_only":read_only}));
        }
        if self.call("transact", json!(operations))?["ok"] != true {
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
