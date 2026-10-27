use crate::ast::{BinaryOp, Expr, Literal, Program, Stmt, UnaryOp};
use crate::bytecode::{Chunk, Constant, OpCode};
use crate::diagnostics::Diagnostic;

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, Vec<Diagnostic>> {
        for stmt in &program.statements {
            self.compile_stmt(stmt);
        }

        // Final halt opcode
        let final_line = self.chunk.lines.last().copied().unwrap_or(1);
        self.chunk.write_opcode(OpCode::OpHalt, final_line);

        if self.diagnostics.is_empty() {
            Ok(self.chunk)
        } else {
            Err(self.diagnostics)
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.chunk.write_opcode(OpCode::OpPop, line);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    fn emit_jump(&mut self, opcode: OpCode, line: usize) -> usize {
        self.chunk.write_opcode(opcode, line);
        self.chunk.write_u16(0xFFFF, line);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump_target = self.chunk.code.len();
        if jump_target > u16::MAX as usize {
            self.diagnostics.push(Diagnostic::error(
                "bytecode jump target exceeds maximum offset (65535)",
                None,
            ));
            return;
        }
        let target = jump_target as u16;
        self.chunk.code[offset] = (target >> 8) as u8;
        self.chunk.code[offset + 1] = (target & 0xFF) as u8;
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                initializer,
                span,
                ..
            } => {
                let line = span.start.line;
                self.compile_expr(initializer);

                if self.scope_depth == 0 {
                    let const_idx = self.chunk.add_constant(Constant::String(name.clone()));
                    self.chunk.write_opcode(OpCode::OpDefGlobal, line);
                    self.chunk.write_u16(const_idx, line);
                } else {
                    self.locals.push(Local {
                        name: name.clone(),
                        depth: self.scope_depth,
                    });
                }
            }

            Stmt::Expression(expr, _) => {
                let line = expr.span().start.line;
                self.compile_expr(expr);
                self.chunk.write_opcode(OpCode::OpPop, line);
            }

            Stmt::Print(expr, span) => {
                let line = span.start.line;
                self.compile_expr(expr);
                self.chunk.write_opcode(OpCode::OpPrint, line);
            }

            Stmt::Println(expr, span) => {
                let line = span.start.line;
                self.compile_expr(expr);
                self.chunk.write_opcode(OpCode::OpPrintln, line);
            }

            Stmt::Block(stmts, span) => {
                self.begin_scope();
                for s in stmts {
                    self.compile_stmt(s);
                }
                self.end_scope(span.end.line);
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        let line = expr.span().start.line;

        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(n) => {
                    let idx = self.chunk.add_constant(Constant::Int(*n));
                    self.chunk.write_opcode(OpCode::OpConstant, line);
                    self.chunk.write_u16(idx, line);
                }
                Literal::Float(n) => {
                    let idx = self.chunk.add_constant(Constant::Float(*n));
                    self.chunk.write_opcode(OpCode::OpConstant, line);
                    self.chunk.write_u16(idx, line);
                }
                Literal::String(s) => {
                    let idx = self.chunk.add_constant(Constant::String(s.clone()));
                    self.chunk.write_opcode(OpCode::OpConstant, line);
                    self.chunk.write_u16(idx, line);
                }
                Literal::Bool(true) => {
                    self.chunk.write_opcode(OpCode::OpTrue, line);
                }
                Literal::Bool(false) => {
                    self.chunk.write_opcode(OpCode::OpFalse, line);
                }
                Literal::Nil => {
                    self.chunk.write_opcode(OpCode::OpNil, line);
                }
            },

            Expr::Variable(name, _) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write_opcode(OpCode::OpGetLocal, line);
                    self.chunk.write_u16(slot as u16, line);
                } else {
                    let const_idx = self.chunk.add_constant(Constant::String(name.clone()));
                    self.chunk.write_opcode(OpCode::OpGetGlobal, line);
                    self.chunk.write_u16(const_idx, line);
                }
            }

            Expr::Assign(name, val_expr, _) => {
                self.compile_expr(val_expr);
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write_opcode(OpCode::OpSetLocal, line);
                    self.chunk.write_u16(slot as u16, line);
                } else {
                    let const_idx = self.chunk.add_constant(Constant::String(name.clone()));
                    self.chunk.write_opcode(OpCode::OpSetGlobal, line);
                    self.chunk.write_u16(const_idx, line);
                }
            }

            Expr::Unary(op, operand, _) => {
                self.compile_expr(operand);
                match op {
                    UnaryOp::Negate => self.chunk.write_opcode(OpCode::OpNegate, line),
                    UnaryOp::Not => self.chunk.write_opcode(OpCode::OpNot, line),
                }
            }

            Expr::Binary(left, op, right, _) => match op {
                BinaryOp::And => {
                    self.compile_expr(left);
                    let end_jump = self.emit_jump(OpCode::OpJumpIfFalse, line);
                    self.chunk.write_opcode(OpCode::OpPop, line);
                    self.compile_expr(right);
                    self.patch_jump(end_jump);
                }
                BinaryOp::Or => {
                    self.compile_expr(left);
                    let else_jump = self.emit_jump(OpCode::OpJumpIfFalse, line);
                    let end_jump = self.emit_jump(OpCode::OpJump, line);
                    self.patch_jump(else_jump);
                    self.chunk.write_opcode(OpCode::OpPop, line);
                    self.compile_expr(right);
                    self.patch_jump(end_jump);
                }
                _ => {
                    self.compile_expr(left);
                    self.compile_expr(right);
                    match op {
                        BinaryOp::Add => self.chunk.write_opcode(OpCode::OpAdd, line),
                        BinaryOp::Subtract => self.chunk.write_opcode(OpCode::OpSubtract, line),
                        BinaryOp::Multiply => self.chunk.write_opcode(OpCode::OpMultiply, line),
                        BinaryOp::Divide => self.chunk.write_opcode(OpCode::OpDivide, line),
                        BinaryOp::Modulo => self.chunk.write_opcode(OpCode::OpModulo, line),
                        BinaryOp::Equal => self.chunk.write_opcode(OpCode::OpEqual, line),
                        BinaryOp::NotEqual => self.chunk.write_opcode(OpCode::OpNotEqual, line),
                        BinaryOp::Less => self.chunk.write_opcode(OpCode::OpLess, line),
                        BinaryOp::LessEqual => self.chunk.write_opcode(OpCode::OpLessEqual, line),
                        BinaryOp::Greater => self.chunk.write_opcode(OpCode::OpGreater, line),
                        BinaryOp::GreaterEqual => {
                            self.chunk.write_opcode(OpCode::OpGreaterEqual, line)
                        }
                        BinaryOp::And | BinaryOp::Or => unreachable!(),
                    }
                }
            },

            Expr::Grouping(inner, _) => {
                self.compile_expr(inner);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    #[test]
    fn test_compiler_bytecode_generation() {
        let code = "val a = 5\nval b = 10\nprintln(a + b)";
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        SemanticAnalyzer::new().analyze(&program).unwrap();

        let chunk = Compiler::new().compile(&program).unwrap();
        assert!(!chunk.code.is_empty());
        assert_eq!(*chunk.code.last().unwrap(), OpCode::OpHalt as u8);
    }
}
