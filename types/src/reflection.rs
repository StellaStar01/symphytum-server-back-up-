use std::collections::HashMap;
use std::fmt::Write;
use std::sync::LazyLock;

use prost_reflect::{FieldDescriptor, Kind, MapKey, ReflectMessage, Value};

pub use prost_reflect::{DescriptorPool, DynamicMessage};

pub static DESCRIPTOR_POOL: LazyLock<DescriptorPool> = LazyLock::new(|| {
    DescriptorPool::decode(
        include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin")).as_ref(),
    )
    .expect("failed to decode descriptor set")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    UnknownMessage(String),
    Decode(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnknownMessage(name) => write!(f, "unknown message: {name}"),
            DecodeError::Decode(msg) => write!(f, "decode error: {msg}"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub fn decode_pretty(name: &str, bytes: &[u8]) -> Result<String, DecodeError> {
    let desc = DESCRIPTOR_POOL
        .get_message_by_name(name)
        .ok_or_else(|| DecodeError::UnknownMessage(name.to_owned()))?;
    let msg =
        DynamicMessage::decode(desc, bytes).map_err(|e| DecodeError::Decode(e.to_string()))?;
    Ok(format_message(&msg))
}

pub fn format_message(msg: &DynamicMessage) -> String {
    let mut out = String::new();
    write_message(&mut out, msg, 0);
    out
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("    ");
    }
}

fn write_message(out: &mut String, msg: &DynamicMessage, indent: usize) {
    let name = msg.descriptor().name().to_owned();
    let mut first = true;

    for field in msg.descriptor().fields() {
        if field.containing_oneof().is_some() && !msg.has_field(&field) {
            continue;
        }

        if first {
            let _ = write!(out, "{name} {{");
            first = false;
        }

        let _ = write!(out, "\n");
        write_indent(out, indent + 1);
        let _ = write!(out, "{}: ", field.name());

        if matches!(field.kind(), Kind::Message(_)) && !field.is_list() && !field.is_map() {
            if msg.has_field(&field) {
                out.push_str("Some(");
                write_value(out, Some(&field), &msg.get_field(&field), indent + 1);
                out.push(')');
            } else {
                out.push_str("None");
            }
        } else {
            write_value(out, Some(&field), &msg.get_field(&field), indent + 1);
        }
        out.push(',');
    }
    if first {
        let _ = write!(out, "{name} {{}}");
    } else {
        let _ = write!(out, "\n");
        write_indent(out, indent);
        out.push('}');
    }
}

fn write_value(out: &mut String, field: Option<&FieldDescriptor>, value: &Value, indent: usize) {
    match value {
        Value::Message(m) => write_message(out, m, indent),
        Value::List(items) => write_list(out, field, items, indent),
        Value::Map(entries) => write_map(out, field, entries, indent),
        Value::EnumNumber(n) => match field.and_then(enum_descriptor_of) {
            Some(ed) => match ed.get_value(*n) {
                Some(v) => out.push_str(v.name()),
                None => {
                    let _ = write!(out, "{n}");
                }
            },
            None => {
                let _ = write!(out, "{n}");
            }
        },
        Value::Bytes(b) => match std::str::from_utf8(b.as_ref()) {
            Ok(s) => {
                let _ = write!(out, "{s:?}");
            }
            Err(_) => {
                let _ = write!(out, "0x{}", hex(b.as_ref()));
            }
        },
        Value::String(s) => {
            let _ = write!(out, "{s:?}");
        }
        Value::Bool(b) => {
            let _ = write!(out, "{b}");
        }
        Value::I32(v) => {
            let _ = write!(out, "{v}");
        }
        Value::I64(v) => {
            let _ = write!(out, "{v}");
        }
        Value::U32(v) => {
            let _ = write!(out, "{v}");
        }
        Value::U64(v) => {
            let _ = write!(out, "{v}");
        }
        Value::F32(v) => {
            let _ = write!(out, "{v}");
        }
        Value::F64(v) => {
            let _ = write!(out, "{v}");
        }
    }
}

fn enum_descriptor_of(field: &FieldDescriptor) -> Option<prost_reflect::EnumDescriptor> {
    match field.kind() {
        Kind::Enum(ed) => Some(ed),
        _ => None,
    }
}

fn write_list(out: &mut String, field: Option<&FieldDescriptor>, items: &[Value], indent: usize) {
    if items.is_empty() {
        out.push_str("[]");
        return;
    }

    let multi = items.iter().any(|v| matches!(v, Value::Message(_)));
    if !multi {
        out.push('[');
        for (i, v) in items.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            write_value(out, field, v, indent);
        }
        out.push(']');
    } else {
        out.push('[');
        for v in items {
            let _ = write!(out, "\n");
            write_indent(out, indent + 1);
            write_value(out, field, v, indent + 1);
            out.push(',');
        }
        let _ = write!(out, "\n");
        write_indent(out, indent);
        out.push(']');
    }
}

fn write_map(
    out: &mut String,
    field: Option<&FieldDescriptor>,
    entries: &HashMap<MapKey, Value>,
    indent: usize,
) {
    if entries.is_empty() {
        out.push_str("{}");
        return;
    }

    let mut sorted: Vec<(String, &Value)> = entries
        .iter()
        .map(|(k, v)| (map_key_string(k), v))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let value_field = field.and_then(|f| match f.kind() {
        Kind::Message(md) if md.is_map_entry() => Some(md.map_entry_value_field()),
        _ => None,
    });

    let multi = entries.values().any(|v| matches!(v, Value::Message(_)));
    if !multi {
        out.push('{');
        for (i, (k, v)) in sorted.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{k}: ");
            write_value(out, value_field.as_ref(), v, indent);
        }
        out.push('}');
    } else {
        out.push('{');
        for (k, v) in sorted {
            let _ = write!(out, "\n");
            write_indent(out, indent + 1);
            let _ = write!(out, "{k}: ");
            write_value(out, value_field.as_ref(), v, indent + 1);
            out.push(',');
        }
        let _ = write!(out, "\n");
        write_indent(out, indent);
        out.push('}');
    }
}

fn map_key_string(key: &MapKey) -> String {
    match key {
        MapKey::String(s) => format!("{s:?}"),
        MapKey::Bool(b) => b.to_string(),
        MapKey::I32(v) => v.to_string(),
        MapKey::I64(v) => v.to_string(),
        MapKey::U32(v) => v.to_string(),
        MapKey::U64(v) => v.to_string(),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}
