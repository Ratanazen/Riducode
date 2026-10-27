use super::opcode::OpCode;
use std::fmt;

pub const MAGIC_HEADER: [u8; 4] = [0x52, 0x49, 0x44, 0x55]; // "RIDU"
pub const BYTECODE_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(n) => write!(f, "{}", n),
            Constant::Float(n) => write!(f, "{}", n),
            Constant::String(s) => write!(f, "\"{}\"", s),
            Constant::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write_byte(opcode as u8, line);
    }

    pub fn write_u16(&mut self, value: u16, line: usize) {
        self.write_byte((value >> 8) as u8, line);
        self.write_byte((value & 0xFF) as u8, line);
    }

    pub fn add_constant(&mut self, value: Constant) -> u16 {
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return i as u16;
            }
        }
        let index = self.constants.len();
        self.constants.push(value);
        index as u16
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // 1. Magic
        bytes.extend_from_slice(&MAGIC_HEADER);
        // 2. Version
        bytes.extend_from_slice(&BYTECODE_VERSION.to_be_bytes());
        // 3. Constant pool count
        let const_count = self.constants.len() as u32;
        bytes.extend_from_slice(&const_count.to_be_bytes());
        // 4. Constants
        for c in &self.constants {
            match c {
                Constant::Int(n) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&n.to_be_bytes());
                }
                Constant::Float(n) => {
                    bytes.push(2);
                    bytes.extend_from_slice(&n.to_be_bytes());
                }
                Constant::String(s) => {
                    bytes.push(3);
                    let s_bytes = s.as_bytes();
                    let len = s_bytes.len() as u32;
                    bytes.extend_from_slice(&len.to_be_bytes());
                    bytes.extend_from_slice(s_bytes);
                }
                Constant::Bool(b) => {
                    bytes.push(4);
                    bytes.push(if *b { 1 } else { 0 });
                }
            }
        }
        // 5. Code length and code
        let code_len = self.code.len() as u32;
        bytes.extend_from_slice(&code_len.to_be_bytes());
        bytes.extend_from_slice(&self.code);
        // 6. Lines
        for l in &self.lines {
            let line_num = *l as u32;
            bytes.extend_from_slice(&line_num.to_be_bytes());
        }

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 10 {
            return Err("bytecode payload too small to be valid Ridu binary".into());
        }
        if &bytes[0..4] != MAGIC_HEADER {
            return Err("invalid magic header; not a Ridu bytecode binary".into());
        }
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        if version != BYTECODE_VERSION {
            return Err(format!(
                "unsupported bytecode version {}; expected {}",
                version, BYTECODE_VERSION
            ));
        }

        let mut offset = 6;
        let const_count = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            if offset >= bytes.len() {
                return Err("corrupted constant pool in bytecode".into());
            }
            let tag = bytes[offset];
            offset += 1;
            match tag {
                1 => {
                    if offset + 8 > bytes.len() {
                        return Err("truncated integer constant".into());
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&bytes[offset..offset + 8]);
                    offset += 8;
                    constants.push(Constant::Int(i64::from_be_bytes(b)));
                }
                2 => {
                    if offset + 8 > bytes.len() {
                        return Err("truncated float constant".into());
                    }
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&bytes[offset..offset + 8]);
                    offset += 8;
                    constants.push(Constant::Float(f64::from_be_bytes(b)));
                }
                3 => {
                    if offset + 4 > bytes.len() {
                        return Err("truncated string length".into());
                    }
                    let len = u32::from_be_bytes([
                        bytes[offset],
                        bytes[offset + 1],
                        bytes[offset + 2],
                        bytes[offset + 3],
                    ]) as usize;
                    offset += 4;
                    if offset + len > bytes.len() {
                        return Err("truncated string content".into());
                    }
                    let s = String::from_utf8(bytes[offset..offset + len].to_vec())
                        .map_err(|e| format!("invalid UTF-8 string in constant pool: {}", e))?;
                    offset += len;
                    constants.push(Constant::String(s));
                }
                4 => {
                    if offset >= bytes.len() {
                        return Err("truncated bool constant".into());
                    }
                    let b = bytes[offset] != 0;
                    offset += 1;
                    constants.push(Constant::Bool(b));
                }
                other => return Err(format!("unknown constant tag {}", other)),
            }
        }

        if offset + 4 > bytes.len() {
            return Err("truncated code stream length".into());
        }
        let code_len = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        offset += 4;

        if offset + code_len > bytes.len() {
            return Err("truncated code payload".into());
        }
        let code = bytes[offset..offset + code_len].to_vec();
        offset += code_len;

        let mut lines = Vec::with_capacity(code_len);
        for _ in 0..code_len {
            if offset + 4 > bytes.len() {
                break;
            }
            let line = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;
            lines.push(line);
        }

        while lines.len() < code.len() {
            lines.push(1);
        }

        Ok(Chunk {
            code,
            constants,
            lines,
        })
    }
}
