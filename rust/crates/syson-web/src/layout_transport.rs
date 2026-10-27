// SPDX-License-Identifier: EPL-2.0
//! Count borrowed geometry twice (wire and quoted ABI) before allocating output.
//! Immutable frames supply identical fields to the counting and emitting passes.
use super::{BYTE_LIMIT, Frame};
use std::io::{self, Write};
use syson_model::Document;
const ENVELOPE: usize = 2048;
fn budget() -> String {
    "Canvas JSON exceeds the default file/ABI byte budget.".into()
}
struct Count {
    bytes: usize,
    quoted: usize,
}
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for byte in bytes {
            self.bytes += 1;
            self.quoted += match byte {
                b'"' | b'\\' | b'\x08' | b'\t' | b'\n' | b'\x0c' | b'\r' => 2,
                0..=31 => 6,
                _ => 1,
            };
            if self.bytes > BYTE_LIMIT || self.quoted > BYTE_LIMIT - ENVELOPE {
                return Err(io::Error::other("canvas byte budget"));
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Output {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit - self.bytes.len() {
            return Err(io::Error::other("canvas count changed"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
// Match the original compact serde_json map order, including float, null and
// Option spellings, without constructing a second model/geometry Value tree.
fn emit(frame: &Frame, out: &mut impl Write) -> io::Result<()> {
    macro_rules! raw {
        ($bytes:expr) => {
            out.write_all($bytes)?
        };
    }
    macro_rules! value {
        ($value:expr) => {
            serde_json::to_writer(&mut *out, &$value).map_err(io::Error::other)?
        };
    }
    raw!(b"{\"edges\":[");
    for (index, edge) in frame.scene.edges.iter().enumerate() {
        if index != 0 {
            raw!(b",");
        }
        raw!(b"{\"id\":");
        value!(edge.id);
        raw!(b",\"points\":");
        if let Some(points) = edge.data.as_ref().and_then(|d| d.bending_points.as_ref()) {
            raw!(b"[");
            for (index, point) in points.iter().enumerate() {
                if index != 0 {
                    raw!(b",");
                }
                raw!(b"[");
                value!(point.x);
                raw!(b",");
                value!(point.y);
                raw!(b"]");
            }
            raw!(b"]");
        } else {
            raw!(b"null");
        }
        raw!(b",\"source\":");
        value!(edge.source);
        raw!(b",\"target\":");
        value!(edge.target);
        raw!(b"}");
    }
    raw!(b"],\"model\":");
    value!(frame.model.as_str());
    raw!(b",\"nodes\":[");
    for (index, node) in frame.scene.nodes.iter().enumerate() {
        if index != 0 {
            raw!(b",");
        }
        raw!(b"{\"header\":");
        value!(node.payload.header);
        raw!(b",\"height\":");
        value!(node.height);
        raw!(b",\"id\":");
        value!(node.id);
        raw!(b",\"parent\":");
        value!(node.payload.parent);
        raw!(b",\"resized\":");
        value!(node.resized_by_user);
        raw!(b",\"width\":");
        value!(node.width);
        raw!(b",\"x\":");
        value!(node.position.x);
        raw!(b",\"y\":");
        value!(node.position.y);
        raw!(b"}");
    }
    raw!(b"],\"version\":1}");
    Ok(())
}
pub(super) fn prepare(frame: &Frame) -> Result<String, String> {
    let mut count = Count {
        bytes: 0,
        quoted: 2,
    };
    emit(frame, &mut count).map_err(|_| budget())?;
    let mut output = Output {
        bytes: Vec::new(),
        limit: count.bytes,
    };
    output
        .bytes
        .try_reserve_exact(count.bytes)
        .map_err(|_| budget())?;
    emit(frame, &mut output).map_err(|_| budget())?;
    if output.bytes.len() != count.bytes {
        return Err(budget());
    }
    String::from_utf8(output.bytes).map_err(|_| budget())
}
struct Compare<'a> {
    expected: &'a [u8],
    offset: usize,
    equal: bool,
}
impl Write for Compare<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .offset
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("model size"))?;
        self.equal &= self.expected.get(self.offset..end) == Some(bytes);
        self.offset = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn matches_model(doc: &Document, canonical: &str) -> Result<bool, String> {
    let mut compare = Compare {
        expected: canonical.as_bytes(),
        offset: 0,
        equal: true,
    };
    doc.write_json(&mut compare, BYTE_LIMIT, || false)
        .map_err(|error| error.to_string())?;
    Ok(compare.equal && compare.offset == canonical.len())
}
