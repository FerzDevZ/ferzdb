//! Row serialization and deserialization
//! 
//! Rows are stored as contiguous bytes. Fixed-size types are written directly,
//! while variable-length types (Text, Blob) include a length prefix.

use crate::util::types::Value;
use crate::util::error::{FerzError, Result};

/// Row header for MVCC and logging
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RowHeader {
    pub xid: u64,           // Transaction ID that created/modified this version
    pub rollback_ptr: u64,  // Pointer to previous version in Undo Log (0 if none)
}

impl RowHeader {
    pub const SIZE: usize = 16; // 8 bytes XID + 8 bytes Rollback

    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..8].copy_from_slice(&self.xid.to_be_bytes());
        bytes[8..16].copy_from_slice(&self.rollback_ptr.to_be_bytes());
        bytes
    }

    pub fn deserialize(data: &[u8]) -> Self {
        let xid = u64::from_be_bytes(data[0..8].try_into().unwrap());
        let rollback_ptr = u64::from_be_bytes(data[8..16].try_into().unwrap());
        Self { xid, rollback_ptr }
    }
}

pub struct RowSerializer;

impl RowSerializer {
    /// Serialize a list of values into bytes with MVCC header
    pub fn serialize(header: &RowHeader, values: &[Value]) -> Vec<u8> {
        let mut buffer = Vec::new();
        // 1. Add MVCC Header
        buffer.extend_from_slice(&header.serialize());
        
        // 2. Add Values
        for value in values {
            match value {
                Value::Integer(i) => {
                    buffer.push(1); // Type tag
                    buffer.extend_from_slice(&i.to_be_bytes());
                }
                Value::Float(f) => {
                    buffer.push(2);
                    buffer.extend_from_slice(&f.to_be_bytes());
                }
                Value::Text(s) => {
                    buffer.push(3);
                    let bytes = s.as_bytes();
                    buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                    buffer.extend_from_slice(bytes);
                }
                Value::Boolean(b) => {
                    buffer.push(4);
                    buffer.push(if *b { 1 } else { 0 });
                }
                Value::Null => {
                    buffer.push(0);
                }
                Value::Blob(b) => {
                    buffer.push(5);
                    buffer.extend_from_slice(&(b.len() as u32).to_be_bytes());
                    buffer.extend_from_slice(b);
                }
            }
        }
        buffer
    }

    /// Deserialize bytes back into header and values
    pub fn deserialize(data: &[u8]) -> Result<(RowHeader, Vec<Value>)> {
        if data.len() < RowHeader::SIZE {
            return Err(FerzError::Corruption("Row too short for header".to_string()));
        }

        let header = RowHeader::deserialize(&data[0..RowHeader::SIZE]);
        let mut values = Vec::new();
        let mut pos = RowHeader::SIZE;

        while pos < data.len() {
            let tag = data[pos];
            pos += 1;

            match tag {
                0 => values.push(Value::Null),
                1 => {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[pos..pos+8]);
                    values.push(Value::Integer(i64::from_be_bytes(bytes)));
                    pos += 8;
                }
                2 => {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[pos..pos+8]);
                    values.push(Value::Float(f64::from_be_bytes(bytes)));
                    pos += 8;
                }
                3 => {
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&data[pos..pos+4]);
                    let len = u32::from_be_bytes(len_bytes) as usize;
                    pos += 4;
                    let s = String::from_utf8(data[pos..pos+len].to_vec())
                        .map_err(|_| FerzError::Corruption("Invalid UTF-8 in row".to_string()))?;
                    values.push(Value::Text(s));
                    pos += len;
                }
                4 => {
                    values.push(Value::Boolean(data[pos] != 0));
                    pos += 1;
                }
                5 => {
                    let mut len_bytes = [0u8; 4];
                    len_bytes.copy_from_slice(&data[pos..pos+4]);
                    let len = u32::from_be_bytes(len_bytes) as usize;
                    pos += 4;
                    values.push(Value::Blob(data[pos..pos+len].to_vec()));
                    pos += len;
                }
                _ => return Err(FerzError::Corruption(format!("Unknown type tag: {}", tag))),
            }
        }
        Ok((header, values))
    }

    pub fn deserialize_value(data: &[u8]) -> Result<Value> {
        if data.is_empty() {
            return Err(FerzError::Corruption("Empty data for value".to_string()));
        }
        let tag = data[0];
        let mut pos = 1;
        match tag {
            0 => Ok(Value::Null),
            1 => {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[pos..pos+8]);
                Ok(Value::Integer(i64::from_be_bytes(bytes)))
            }
            2 => {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[pos..pos+8]);
                Ok(Value::Float(f64::from_be_bytes(bytes)))
            }
            3 => {
                let mut len_bytes = [0u8; 4];
                len_bytes.copy_from_slice(&data[pos..pos+4]);
                let len = u32::from_be_bytes(len_bytes) as usize;
                pos += 4;
                let s = String::from_utf8(data[pos..pos+len].to_vec())
                    .map_err(|_| FerzError::Corruption("Invalid UTF-8".to_string()))?;
                Ok(Value::Text(s))
            }
            4 => Ok(Value::Boolean(data[pos] != 0)),
            5 => {
                let mut len_bytes = [0u8; 4];
                len_bytes.copy_from_slice(&data[pos..pos+4]);
                let len = u32::from_be_bytes(len_bytes) as usize;
                pos += 4;
                Ok(Value::Blob(data[pos..pos+len].to_vec()))
            }
            _ => Err(FerzError::Corruption(format!("Unknown type tag: {}", tag))),
        }
    }

    pub fn serialize_value(value: &Value) -> Vec<u8> {
        let mut buffer = Vec::new();
        match value {
            Value::Integer(i) => {
                buffer.push(1);
                buffer.extend_from_slice(&i.to_be_bytes());
            }
            Value::Float(f) => {
                buffer.push(2);
                buffer.extend_from_slice(&f.to_be_bytes());
            }
            Value::Text(s) => {
                buffer.push(3);
                let bytes = s.as_bytes();
                buffer.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                buffer.extend_from_slice(bytes);
            }
            Value::Boolean(b) => {
                buffer.push(4);
                buffer.push(if *b { 1 } else { 0 });
            }
            Value::Null => {
                buffer.push(0);
            }
            Value::Blob(b) => {
                buffer.push(5);
                buffer.extend_from_slice(&(b.len() as u32).to_be_bytes());
                buffer.extend_from_slice(b);
            }
        }
        buffer
    }
}

pub struct Row {
    pub header: RowHeader,
    pub values: Vec<Value>,
}
