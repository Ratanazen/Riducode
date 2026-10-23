use super::chunk::Chunk;
use super::opcode::OpCode;

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(chunk: &Chunk, name: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("=== Bytecode Chunk: {} ===\n", name));
        out.push_str("Offset Line Opcode                     Operands / Details\n");
        out.push_str("-----------------------------------------------------------\n");

        let mut offset = 0;
        while offset < chunk.code.len() {
            offset = Self::disassemble_instruction(chunk, offset, &mut out);
        }

        out
    }

    pub fn disassemble_instruction(chunk: &Chunk, offset: usize, out: &mut String) -> usize {
        let line = if offset < chunk.lines.len() {
            chunk.lines[offset]
        } else {
            0
        };

        let line_str = if offset > 0
            && offset < chunk.lines.len()
            && chunk.lines[offset] == chunk.lines[offset - 1]
        {
            "   |".to_string()
        } else {
            format!("{:4}", line)
        };

        let byte = chunk.code[offset];
        let opcode = match OpCode::from_u8(byte) {
            Some(op) => op,
            None => {
                out.push_str(&format!(
                    "{:04}  {}  UNKNOWN_OP(0x{:02X})\n",
                    offset, line_str, byte
                ));
                return offset + 1;
            }
        };

        match opcode {
            OpCode::OpConstant => {
                let (const_idx, next_offset) = Self::read_u16(chunk, offset + 1);
                let const_val = chunk
                    .constants
                    .get(const_idx as usize)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!(
                    "{:04}  {}  {:<26} #{:<4} ({})\n",
                    offset,
                    line_str,
                    opcode.to_string(),
                    const_idx,
                    const_val
                ));
                next_offset
            }
            OpCode::OpDefGlobal | OpCode::OpGetGlobal | OpCode::OpSetGlobal => {
                let (const_idx, next_offset) = Self::read_u16(chunk, offset + 1);
                let const_val = chunk
                    .constants
                    .get(const_idx as usize)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!(
                    "{:04}  {}  {:<26} #{:<4} ({})\n",
                    offset,
                    line_str,
                    opcode.to_string(),
                    const_idx,
                    const_val
                ));
                next_offset
            }
            OpCode::OpGetLocal | OpCode::OpSetLocal => {
                let (slot, next_offset) = Self::read_u16(chunk, offset + 1);
                out.push_str(&format!(
                    "{:04}  {}  {:<26} slot={}\n",
                    offset,
                    line_str,
                    opcode.to_string(),
                    slot
                ));
                next_offset
            }
            OpCode::OpJumpIfFalse | OpCode::OpJump => {
                let (target, next_offset) = Self::read_u16(chunk, offset + 1);
                out.push_str(&format!(
                    "{:04}  {}  {:<26} -> {:04}\n",
                    offset,
                    line_str,
                    opcode.to_string(),
                    target
                ));
                next_offset
            }
            _ => {
                out.push_str(&format!("{:04}  {}  {}\n", offset, line_str, opcode));
                offset + 1
            }
        }
    }

    fn read_u16(chunk: &Chunk, offset: usize) -> (u16, usize) {
        if offset + 1 < chunk.code.len() {
            let high = chunk.code[offset] as u16;
            let low = chunk.code[offset + 1] as u16;
            ((high << 8) | low, offset + 2)
        } else {
            (0, chunk.code.len())
        }
    }
}
