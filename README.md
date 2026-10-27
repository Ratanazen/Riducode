<p align="center">
  <img src="assets/logo.png" width="180" alt="Ridu Programming Language Logo" />
</p>

<h1 align="center">Ridu Programming Language</h1>

<p align="center">
  <strong>A modern, general-purpose programming language featuring Kotlin-inspired readability, static typing, its own compiler pipeline, custom bytecode instruction set, and dedicated stack-based virtual machine.</strong>
</p>

<p align="center">
  <a href="#1-project-overview">Overview</a> •
  <a href="#2-implementation-status-phase-1-compiler-mvp">Status</a> •
  <a href="#3-getting-started">Getting Started</a> •
  <a href="#4-usage--examples">Usage</a> •
  <a href="#5-language-syntax-examples">Syntax</a> •
  <a href="#6-visual-studio-code-extension">VS Code Extension</a> •
  <a href="#8-roadmap-toward-self-hosting">Roadmap</a>
</p>

---

## 1. Project Overview

**Ridu** is an original programming language engineered from the ground up to combine expressive, human-friendly syntax with predictable runtime performance and robust backend development capabilities.

Rust is used solely as the initial bootstrap implementation language. Ridu is **not** a wrapper around another language runtime; it compiles its own Abstract Syntax Tree into its own custom bytecode binary format and executes on its own virtual machine.

### Architecture Pipeline

```text
Source Code (.ridu)
         │
         ▼
  [ Lexer / Scanner ]        ── Token stream with line & column spans
         │
         ▼
  [ Recursive Descent Parser ]── AST (Expressions, Statements, Declarations)
         │
         ▼
  [ Semantic Analyzer ]       ── Scope tables, mutability checks, type checks
         │
         ▼
  [ Bytecode Compiler ]       ── Custom binary Chunk & Constant Pool
         │
         ▼
  [ Ridu Virtual Machine ]    ── Custom stack-based VM with opcode dispatch
         │
         ▼
    Program Output
```

---

## 2. Implementation Status: Phase 1 Compiler MVP

### What is Implemented Today (Phase 1)
- [x] **Source File Loading & Error Formatting**: Clean diagnostic messages with line numbers, column numbers, and source context carets (`file.ridu:line:col: error: ...`).
- [x] **Lexer**: Tokenization of identifiers, keywords, integer literals, float literals, string literals (with escape sequences `\n`, `\t`, `\r`, `\\`, `\"`, `\0`), operators, delimiters, single-line (`//`) and multi-line (`/* ... */`) comments.
- [x] **Parser & AST**: Pratt parsing for expressions (respecting arithmetic, relational, and logical precedence) and recursive descent for statements and declarations.
- [x] **Explicit Mutability Semantics**:
  - `val`: Immutable binding. Reassignment is caught and rejected during semantic analysis.
  - `var`: Mutable binding. Can be updated at runtime.
- [x] **Lexical Scoping & Shadowing**: Block statements `{ ... }` create new lexical environments. Stack slots are reclaimed on scope exit.
- [x] **Type Annotations & Checking**: Optional type signatures (`val x: Int = 10`) with validation against initializer expressions.
- [x] **Expression Evaluation**:
  - Arithmetic: `+`, `-`, `*`, `/`, `%`
  - Relational: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - Logical: `&&`, `||`, `!`
  - String concatenation with automatic type coercion
- [x] **Custom Bytecode Format**: Binary chunk format with magic header `RIDU` (0x52 0x49 0x44 0x55), format versioning, constant pool, bytecode instruction stream, and line number table.
- [x] **Bytecode Disassembler**: Human-readable disassembly showing offsets, lines, opcodes, operands, and constant values.
- [x] **Custom Stack-based VM**: Execution engine with operand stack, global variable store, stack underflow guards, and explicit runtime errors.
- [x] **Command-Line Interface**:
  - `ridu run <file>`: Compile and execute.
  - `ridu check <file>`: Validate syntax and semantics without running.
  - `ridu disasm <file>`: Disassemble bytecode into assembly format.
  - `ridu --version` / `ridu --help`.

### What is NOT Yet Implemented (Future Roadmap)
- Functions and user-defined callable procedures (`fun name(...)`) (Phase 2).
- Conditional branching statements (`if / else`) and loops (`while`) in high-level AST (Phase 2).
- Complex object model: classes, data classes, interfaces (Phase 3).
- Standard library modules, File I/O, JSON serialization, and HTTP networking (Phase 4).
- Self-hosting compiler bootstrap written directly in Ridu (Phase 5).

---

## 3. Getting Started

### Prerequisites

To build and run Ridu, install the Rust toolchain (version 1.80+):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify your installation:
```bash
rustc --version
cargo --version
```

### Building Ridu

Clone the repository and build the binary:

```bash
git clone https://github.com/Ratanazen/Riducode.git
cd Riducode
cargo build --release
```

The compiled binary will be located at `target/release/ridu`.

### Running Tests

Execute the automated unit and integration test suite:

```bash
cargo test
```

Check code formatting:
```bash
cargo fmt --all -- --check
```

---

## 4. Usage & Examples

### Running a Program
```bash
cargo run -- run examples/hello.ridu
```
Output:
```text
Hello, Ridu Programming Language!
Welcome to Phase 1: Custom Compiler & Bytecode VM.
```

### Checking for Errors (Static Analysis)
```bash
cargo run -- check examples/variables.ridu
```
Output:
```text
Validation successful: 'examples/variables.ridu' has no errors.
```

### Disassembling Bytecode
```bash
cargo run -- disasm examples/hello.ridu
```
Sample disassembly output:
```text
=== Bytecode Chunk: examples/hello.ridu ===
Offset Line Opcode                     Operands / Details
-----------------------------------------------------------
0000     2  OP_CONSTANT                #0    ("Hello, Ridu Programming Language!")
0003     |  OP_DEF_GLOBAL              #1    ("greeting")
0006     3  OP_GET_GLOBAL              #1    ("greeting")
0009     |  OP_PRINTLN
0010     4  OP_CONSTANT                #2    ("Welcome to Phase 1: Custom Compiler & Bytecode VM.")
0013     |  OP_PRINTLN
0014     4  OP_HALT
```

---

## 5. Language Syntax Examples

### Variables & Mutability
```kotlin
// Immutable variable
val name = "Ridu"
val version: Int = 1

// Mutable variable
var counter = 0
counter = counter + 1

{
    // Scoped block
    val inner = "Inner scope"
    println(inner)
}
// 'inner' is reclaimed here
```

### Expressions & Logic
```kotlin
val a = 20
val b = 5

println("Sum: " + (a + b))
println("Division: " + (a / b))
println("Comparison: " + (a > b))
println("Logical AND: " + (true && (a == 20)))
```

---

## 6. Visual Studio Code Extension

Ridu comes with an official Visual Studio Code editor extension located in [`editors/vscode`](editors/vscode):

- **Syntax Highlighting**: Full TextMate grammar support for `.ridu` files.
- **Language Configuration**: Automatic brackets, commenting, and smart indentation.
- **Code Snippets**: Quick templates for `main`, `val`, `var`, `fun`, `println`, `if`, and `while`.
- **Branded Icon**: Ridu extension icon and file associations.

### Installation
```bash
# Install directly into your local VS Code extensions folder
cp -r editors/vscode ~/.vscode/extensions/ratanazen.ridu-vscode-0.1.0
```
Reload or restart Visual Studio Code to start coding with full syntax highlighting.

---

## 7. Language Documentation

- [Language Specification](docs/specification.md): EBNF grammar, lexical elements, and type system rules.
- [Bytecode & VM Specification](docs/bytecode.md): Binary chunk format, opcode definitions, stack operations, and VM execution semantics.

---

## 8. Roadmap Toward Self-Hosting

1. **Phase 1 (Completed)**: Custom compiler MVP, lexer, parser, AST, semantic analyzer, bytecode generator, stack-based VM, and CLI.
2. **Phase 2 (Language Core)**: Functions, closures, control flow (`if`/`else`, `while`), arrays, and return values.
3. **Phase 3 (Object Model)**: Classes, data classes, inheritance, interfaces, and string interpolation.
4. **Phase 4 (Backend Foundation)**: Standard library (I/O, networking, HTTP server/client, JSON).
5. **Phase 5 (Self-Hosting)**: Porting the Ridu compiler into Ridu source code and verifying a self-hosting bootstrap.

---

## 9. License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)
