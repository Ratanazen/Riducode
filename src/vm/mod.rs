pub mod value;

use crate::bytecode::{Chunk, Constant, OpCode};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
pub use value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error at line {}: {}", self.line, self.message)
    }
}

pub struct VM<'a> {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    output: Option<&'a mut dyn Write>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            output: None,
        }
    }

    pub fn with_output(mut self, output: &'a mut dyn Write) -> Self {
        self.output = Some(output);
        self
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while self.ip < self.chunk.code.len() {
            let current_ip = self.ip;
            let line = self.current_line(current_ip);
            let byte = self.read_byte();

            let opcode = OpCode::from_u8(byte).ok_or_else(|| RuntimeError {
                message: format!("unknown opcode 0x{:02X}", byte),
                line,
            })?;

            match opcode {
                OpCode::OpConstant => {
                    let const_idx = self.read_u16();
                    let constant =
                        self.chunk
                            .constants
                            .get(const_idx as usize)
                            .ok_or_else(|| RuntimeError {
                                message: format!("constant index #{} out of bounds", const_idx),
                                line,
                            })?;
                    let val = self.constant_to_value(constant);
                    self.push(val);
                }

                OpCode::OpNil => self.push(Value::Nil),
                OpCode::OpTrue => self.push(Value::Bool(true)),
                OpCode::OpFalse => self.push(Value::Bool(false)),

                OpCode::OpPop => {
                    self.pop(line)?;
                }

                OpCode::OpDefGlobal => {
                    let const_idx = self.read_u16();
                    let name = self.read_string_constant(const_idx, line)?;
                    let value = self.pop(line)?;
                    self.globals.insert(name, value);
                }

                OpCode::OpGetGlobal => {
                    let const_idx = self.read_u16();
                    let name = self.read_string_constant(const_idx, line)?;
                    let value = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| RuntimeError {
                            message: format!("undefined global variable '{}'", name),
                            line,
                        })?;
                    self.push(value);
                }

                OpCode::OpSetGlobal => {
                    let const_idx = self.read_u16();
                    let name = self.read_string_constant(const_idx, line)?;
                    let value = self.peek(0, line)?.clone();
                    if !self.globals.contains_key(&name) {
                        return Err(RuntimeError {
                            message: format!("cannot set undefined global variable '{}'", name),
                            line,
                        });
                    }
                    self.globals.insert(name, value);
                }

                OpCode::OpGetLocal => {
                    let slot = self.read_u16() as usize;
                    if slot >= self.stack.len() {
                        return Err(RuntimeError {
                            message: format!("local slot index {} out of bounds", slot),
                            line,
                        });
                    }
                    let val = self.stack[slot].clone();
                    self.push(val);
                }

                OpCode::OpSetLocal => {
                    let slot = self.read_u16() as usize;
                    if slot >= self.stack.len() {
                        return Err(RuntimeError {
                            message: format!("local slot index {} out of bounds", slot),
                            line,
                        });
                    }
                    let val = self.peek(0, line)?.clone();
                    self.stack[slot] = val;
                }

                OpCode::OpAdd => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let res = a
                        .add(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpSubtract => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let res = a
                        .subtract(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpMultiply => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let res = a
                        .multiply(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpDivide => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let res = a
                        .divide(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpModulo => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let res = a
                        .modulo(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpNegate => {
                    let a = self.pop(line)?;
                    let res = a
                        .negate()
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(res);
                }

                OpCode::OpNot => {
                    let a = self.pop(line)?;
                    self.push(Value::Bool(!a.is_truthy()));
                }

                OpCode::OpEqual => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    self.push(Value::Bool(a == b));
                }

                OpCode::OpNotEqual => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    self.push(Value::Bool(a != b));
                }

                OpCode::OpGreater => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let ord = a
                        .compare(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(Value::Bool(ord == Ordering::Greater));
                }

                OpCode::OpGreaterEqual => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let ord = a
                        .compare(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(Value::Bool(matches!(
                        ord,
                        Ordering::Greater | Ordering::Equal
                    )));
                }

                OpCode::OpLess => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let ord = a
                        .compare(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(Value::Bool(ord == Ordering::Less));
                }

                OpCode::OpLessEqual => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    let ord = a
                        .compare(&b)
                        .map_err(|err| RuntimeError { message: err, line })?;
                    self.push(Value::Bool(matches!(ord, Ordering::Less | Ordering::Equal)));
                }

                OpCode::OpPrint => {
                    let val = self.pop(line)?;
                    if let Some(ref mut out) = self.output {
                        let _ = write!(out, "{}", val);
                    } else {
                        print!("{}", val);
                    }
                }

                OpCode::OpPrintln => {
                    let val = self.pop(line)?;
                    if let Some(ref mut out) = self.output {
                        let _ = writeln!(out, "{}", val);
                    } else {
                        println!("{}", val);
                    }
                }

                OpCode::OpJumpIfFalse => {
                    let target = self.read_u16() as usize;
                    let val = self.peek(0, line)?;
                    if !val.is_truthy() {
                        self.ip = target;
                    }
                }

                OpCode::OpJump => {
                    let target = self.read_u16() as usize;
                    self.ip = target;
                }

                OpCode::OpHalt => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.chunk.code[self.ip];
        self.ip += 1;
        b
    }

    fn read_u16(&mut self) -> u16 {
        let high = self.chunk.code[self.ip] as u16;
        let low = self.chunk.code[self.ip + 1] as u16;
        self.ip += 2;
        (high << 8) | low
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self, line: usize) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or_else(|| RuntimeError {
            message: "stack underflow".into(),
            line,
        })
    }

    fn peek(&self, distance: usize, line: usize) -> Result<&Value, RuntimeError> {
        if self.stack.len() <= distance {
            return Err(RuntimeError {
                message: "stack underflow on peek".into(),
                line,
            });
        }
        Ok(&self.stack[self.stack.len() - 1 - distance])
    }

    fn current_line(&self, offset: usize) -> usize {
        if offset < self.chunk.lines.len() {
            self.chunk.lines[offset]
        } else {
            1
        }
    }

    fn constant_to_value(&self, c: &Constant) -> Value {
        match c {
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(n) => Value::Float(*n),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Bool(b) => Value::Bool(*b),
        }
    }

    fn read_string_constant(&self, idx: u16, line: usize) -> Result<String, RuntimeError> {
        match self.chunk.constants.get(idx as usize) {
            Some(Constant::String(s)) => Ok(s.clone()),
            _ => Err(RuntimeError {
                message: format!("expected string constant at index #{}", idx),
                line,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_arithmetic() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Constant::Int(15));
        let idx2 = chunk.add_constant(Constant::Int(27));

        chunk.write_opcode(OpCode::OpConstant, 1);
        chunk.write_u16(idx1, 1);
        chunk.write_opcode(OpCode::OpConstant, 1);
        chunk.write_u16(idx2, 1);
        chunk.write_opcode(OpCode::OpAdd, 1);
        chunk.write_opcode(OpCode::OpHalt, 1);

        let mut vm = VM::new(chunk);
        vm.run().unwrap();
        assert_eq!(vm.stack.pop().unwrap(), Value::Int(42));
    }
}
