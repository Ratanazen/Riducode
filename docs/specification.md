# Ridu Language Specification (Phase 1 MVP)

Version: 0.1.0
Status: Active

## 1. Overview

**Ridu** is an original, general-purpose programming language designed with Kotlin-inspired readability, strong static semantics, and clear runtime boundaries.

In Phase 1, Ridu provides a self-contained language frontend (lexer, parser, AST, semantic analyzer) and backend (custom bytecode emitter, disassembler, and custom stack-based bytecode virtual machine).

---

## 2. Lexical Grammar

### 2.1 Whitespace and Comments
- Whitespace (` `, `\t`, `\r`, `\n`) separates tokens.
- Single-line comments start with `//` and extend to the end of the line.
- Multi-line block comments are enclosed between `/*` and `*/`.

### 2.2 Keywords
The following identifiers are reserved keywords in Ridu:
```
val     var     fun     if      else    while   return
print   println true    false   nil
```

### 2.3 Literals
- **Integer**: Sequences of ASCII digits, optionally separated by `_` for readability (e.g., `42`, `1_000_000`).
- **Float**: Digits with a fractional dot (e.g., `3.14`, `0.5`, `10.0`).
- **String**: Enclosed in double quotes `"..."`. Supported escape sequences:
  - `\n` (newline)
  - `\t` (tab)
  - `\r` (carriage return)
  - `\\` (backslash)
  - `\"` (double quote)
  - `\0` (null)
- **Boolean**: `true` and `false`.
- **Nil**: `nil`.

### 2.4 Operators and Delimiters
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment: `=`
- Delimiters: `(`, `)`, `{`, `}`, `:`, `;`, `,`

---

## 3. Syntactic Grammar (EBNF)

```ebnf
Program        ::= Statement* EOF ;

Statement      ::= VarDecl
                 | PrintStmt
                 | PrintlnStmt
                 | Block
                 | ExprStmt ;

VarDecl        ::= ("val" | "var") IDENTIFIER (":" TypeAnnotation)? "=" Expression ";"? ;
PrintStmt      ::= "print" ("(" Expression ")" | Expression) ";"? ;
PrintlnStmt    ::= "println" ("(" Expression ")" | Expression) ";"? ;
Block          ::= "{" Statement* "}" ;
ExprStmt       ::= Expression ";"? ;

TypeAnnotation ::= "Int" | "Float" | "String" | "Bool" | "Unit" ;

Expression     ::= Assignment ;
Assignment     ::= IDENTIFIER "=" Assignment | LogicalOr ;
LogicalOr      ::= LogicalAnd ("||" LogicalAnd)* ;
LogicalAnd     ::= Equality ("&&" Equality)* ;
Equality       ::= Comparison (("==" | "!=") Comparison)* ;
Comparison     ::= Term (("<" | "<=" | ">" | ">=") Term)* ;
Term           ::= Factor (("+" | "-") Factor)* ;
Factor         ::= Unary (("*" | "/" | "%") Unary)* ;
Unary          ::= ("-" | "!") Unary | Primary ;
Primary        ::= INT | FLOAT | STRING | "true" | "false" | "nil"
                 | IDENTIFIER
                 | "(" Expression ")" ;
```

---

## 4. Semantic Rules

### 4.1 Immutability (`val` vs `var`)
- Variables declared with `val` are immutable. Any subsequent assignment to a `val` variable results in a compile-time semantic error.
- Variables declared with `var` are mutable and can be reassigned.

### 4.2 Scope and Shadowing
- A block `{ ... }` introduces a new lexical scope.
- Variables declared inside a block shadow variables with the same name declared in outer scopes.
- Declaring two variables with the identical name in the exact same scope is forbidden and triggers a diagnostic error.

### 4.3 Type Compatibility
- Strict numeric operations:
  - `Int + Int -> Int`
  - `Float + Float -> Float`
  - String concatenation: `String + <Any> -> String`
- Division and modulo by zero trigger explicit runtime errors.
