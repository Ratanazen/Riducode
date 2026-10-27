# Ridu Bytecode & Virtual Machine Specification

Version: 0.1.0
Magic: `RIDU` (0x52 0x49 0x44 0x55)
Bytecode Version: 1

---

## 1. Binary Chunk Layout

A compiled Ridu bytecode binary contains the following structure:

```
+-----------------------------------------------------------+
| Magic Bytes: [0x52, 0x49, 0x44, 0x55] ("RIDU")            | (4 bytes)
+-----------------------------------------------------------+
| Format Version: 0x0001                                    | (2 bytes, big-endian)
+-----------------------------------------------------------+
| Constant Pool Count: N                                    | (4 bytes, big-endian)
+-----------------------------------------------------------+
| Constant Entries:                                         |
|   - Tag (1=Int, 2=Float, 3=String, 4=Bool)                | (1 byte)
|   - Payload (i64, f64, length-prefixed bytes, or u8)      | (variable)
+-----------------------------------------------------------+
| Code Length: M                                            | (4 bytes, big-endian)
+-----------------------------------------------------------+
| Bytecode Stream: [Opcode, Operands...]                    | (M bytes)
+-----------------------------------------------------------+
| Source Line Table:                                        | (M * 4 bytes, big-endian)
+-----------------------------------------------------------+
```

---

## 2. Opcode Table

| Opcode | Hex | Mnemonic | Operands | Stack Effect | Description |
|---|---|---|---|---|---|
| 1 | `0x01` | `OP_CONSTANT` | `u16` (index) | `-> value` | Pushes constant at pool index |
| 2 | `0x02` | `OP_NIL` | none | `-> nil` | Pushes nil |
| 3 | `0x03` | `OP_TRUE` | none | `-> true` | Pushes boolean true |
| 4 | `0x04` | `OP_FALSE` | none | `-> false` | Pushes boolean false |
| 5 | `0x05` | `OP_POP` | none | `value ->` | Discards top of stack |
| 6 | `0x06` | `OP_DEF_GLOBAL` | `u16` (name index) | `value ->` | Pop value and define global symbol |
| 7 | `0x07` | `OP_GET_GLOBAL` | `u16` (name index) | `-> value` | Push global value |
| 8 | `0x08` | `OP_SET_GLOBAL` | `u16` (name index) | `value -> value` | Update global symbol |
| 9 | `0x09` | `OP_GET_LOCAL` | `u16` (slot index) | `-> value` | Push value from stack slot |
| 10 | `0x0A` | `OP_SET_LOCAL` | `u16` (slot index) | `value -> value` | Update value in stack slot |
| 11 | `0x0B` | `OP_ADD` | none | `a, b -> a+b` | Add or string concatenate |
| 12 | `0x0C` | `OP_SUBTRACT` | none | `a, b -> a-b` | Subtract numeric values |
| 13 | `0x0D` | `OP_MULTIPLY` | none | `a, b -> a*b` | Multiply numeric values |
| 14 | `0x0E` | `OP_DIVIDE` | none | `a, b -> a/b` | Divide numeric values |
| 15 | `0x0F` | `OP_MODULO` | none | `a, b -> a%b` | Modulo on numeric values |
| 16 | `0x10` | `OP_NEGATE` | none | `a -> -a` | Negate numeric value |
| 17 | `0x11` | `OP_NOT` | none | `a -> !a` | Logical negation |
| 18 | `0x12` | `OP_EQUAL` | none | `a, b -> a==b` | Test equality |
| 19 | `0x13` | `OP_NOT_EQUAL` | none | `a, b -> a!=b` | Test inequality |
| 20 | `0x14` | `OP_GREATER` | none | `a, b -> a>b` | Test greater than |
| 21 | `0x15` | `OP_GREATER_EQUAL` | none | `a, b -> a>=b` | Test greater than or equal |
| 22 | `0x16` | `OP_LESS` | none | `a, b -> a<b` | Test less than |
| 23 | `0x17` | `OP_LESS_EQUAL` | none | `a, b -> a<=b` | Test less than or equal |
| 24 | `0x18` | `OP_PRINT` | none | `val ->` | Print value to output stream |
| 25 | `0x19` | `OP_PRINTLN` | none | `val ->` | Print value with newline |
| 26 | `0x1A` | `OP_JUMP_IF_FALSE` | `u16` (target offset) | `val -> val` | Jump if top of stack is falsy |
| 27 | `0x1B` | `OP_JUMP` | `u16` (target offset) | none | Unconditional jump |
| 255 | `0xFF` | `OP_HALT` | none | none | Terminates VM execution |

---

## 3. VM Execution Architecture

- **Stack-based VM**:
  - Operands and local variables live in a single contiguous, dynamic stack frame.
  - Safe stack boundary enforcement: underflow checks prevent invalid memory reads.
- **Instruction Pointer (`ip`)**:
  - Traverses the bytecode instruction stream sequentially unless redirected by jump opcodes.
- **Global Variables**:
  - Maintained in an isolated symbol environment keyed by identifier names.
- **Diagnostics and Debugging**:
  - Every bytecode instruction maps 1-to-1 to a source line number for exact stack traces and runtime error messages.
