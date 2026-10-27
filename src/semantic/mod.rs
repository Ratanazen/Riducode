pub mod symbol_table;

use crate::ast::{BinaryOp, Expr, Literal, Program, Stmt, TypeAnnotation, UnaryOp};
use crate::diagnostics::Diagnostic;
use symbol_table::{Symbol, SymbolTable};

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn analyze(mut self, program: &Program) -> Result<(), Vec<Diagnostic>> {
        for stmt in &program.statements {
            self.analyze_stmt(stmt);
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics)
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                is_mutable,
                type_annotation,
                initializer,
                span,
            } => {
                let init_type = self.analyze_expr(initializer);

                if let Some(ref declared) = type_annotation {
                    if let Some(ref init_ty) = init_type {
                        if !self.types_compatible(declared, init_ty) {
                            self.diagnostics.push(Diagnostic::error(
                                format!(
                                    "type mismatch for variable '{}': declared as {}, but initialized with {}",
                                    name, declared, init_ty
                                ),
                                Some(initializer.span()),
                            ));
                        }
                    }
                }

                let final_type = type_annotation.clone().or(init_type);
                let symbol = Symbol {
                    name: name.clone(),
                    is_mutable: *is_mutable,
                    declared_type: type_annotation.clone(),
                    inferred_type: final_type,
                    span: *span,
                    depth: self.symbols.depth(),
                };

                if let Err(existing) = self.symbols.define(symbol) {
                    self.diagnostics.push(Diagnostic::error(
                        format!(
                            "variable '{}' is already declared in this scope (previous declaration at {})",
                            name, existing.span.start
                        ),
                        Some(*span),
                    ));
                }
            }
            Stmt::Expression(expr, _) => {
                self.analyze_expr(expr);
            }
            Stmt::Print(expr, _) | Stmt::Println(expr, _) => {
                self.analyze_expr(expr);
            }
            Stmt::Block(stmts, _) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.analyze_stmt(s);
                }
                self.symbols.exit_scope();
            }
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Option<TypeAnnotation> {
        match expr {
            Expr::Literal(lit, _) => Some(match lit {
                Literal::Int(_) => TypeAnnotation::Int,
                Literal::Float(_) => TypeAnnotation::Float,
                Literal::String(_) => TypeAnnotation::String,
                Literal::Bool(_) => TypeAnnotation::Bool,
                Literal::Nil => TypeAnnotation::Inferred,
            }),

            Expr::Variable(name, span) => match self.symbols.resolve(name) {
                Some(sym) => sym
                    .inferred_type
                    .clone()
                    .or_else(|| sym.declared_type.clone()),
                None => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("cannot find variable '{}' in this scope", name),
                        Some(*span),
                    ));
                    None
                }
            },

            Expr::Assign(name, value, span) => {
                let val_type = self.analyze_expr(value);

                match self.symbols.resolve(name) {
                    Some(sym) => {
                        if !sym.is_mutable {
                            self.diagnostics.push(Diagnostic::error(
                                format!(
                                    "cannot reassign immutable variable '{}' declared with 'val'",
                                    name
                                ),
                                Some(*span),
                            ));
                        } else if let Some(ref sym_ty) = sym
                            .inferred_type
                            .clone()
                            .or_else(|| sym.declared_type.clone())
                        {
                            if let Some(ref r_ty) = val_type {
                                if !self.types_compatible(sym_ty, r_ty) {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!(
                                            "type mismatch in assignment to '{}': expected {}, found {}",
                                            name, sym_ty, r_ty
                                        ),
                                        Some(value.span()),
                                    ));
                                }
                            }
                        }
                    }
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            format!("cannot assign to undeclared variable '{}'", name),
                            Some(*span),
                        ));
                    }
                }

                val_type
            }

            Expr::Unary(op, operand, span) => {
                let operand_type = self.analyze_expr(operand);
                match op {
                    UnaryOp::Negate => {
                        if let Some(ref ty) = operand_type {
                            if *ty != TypeAnnotation::Int
                                && *ty != TypeAnnotation::Float
                                && *ty != TypeAnnotation::Inferred
                            {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("unary '-' cannot be applied to type {}", ty),
                                    Some(*span),
                                ));
                            }
                        }
                        operand_type
                    }
                    UnaryOp::Not => {
                        if let Some(ref ty) = operand_type {
                            if *ty != TypeAnnotation::Bool && *ty != TypeAnnotation::Inferred {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("unary '!' cannot be applied to type {}", ty),
                                    Some(*span),
                                ));
                            }
                        }
                        Some(TypeAnnotation::Bool)
                    }
                }
            }

            Expr::Binary(left, op, right, span) => {
                let left_type = self.analyze_expr(left);
                let right_type = self.analyze_expr(right);

                match op {
                    BinaryOp::Add => {
                        match (&left_type, &right_type) {
                            (Some(TypeAnnotation::Int), Some(TypeAnnotation::Int)) => {
                                Some(TypeAnnotation::Int)
                            }
                            (Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)) => {
                                Some(TypeAnnotation::Float)
                            }
                            (Some(TypeAnnotation::String), Some(TypeAnnotation::String)) => {
                                Some(TypeAnnotation::String)
                            }
                            (Some(TypeAnnotation::String), _)
                            | (_, Some(TypeAnnotation::String)) => {
                                // String concatenation is permitted with automatic string coercion
                                Some(TypeAnnotation::String)
                            }
                            (Some(l), Some(r)) => {
                                if *l != TypeAnnotation::Inferred
                                    && *r != TypeAnnotation::Inferred
                                    && l != r
                                {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!(
                                            "operator '+' cannot be applied to types {} and {}",
                                            l, r
                                        ),
                                        Some(*span),
                                    ));
                                }
                                None
                            }
                            _ => None,
                        }
                    }

                    BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo => match (&left_type, &right_type) {
                        (Some(TypeAnnotation::Int), Some(TypeAnnotation::Int)) => {
                            Some(TypeAnnotation::Int)
                        }
                        (Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)) => {
                            Some(TypeAnnotation::Float)
                        }
                        (Some(l), Some(r)) => {
                            if *l != TypeAnnotation::Inferred && *r != TypeAnnotation::Inferred {
                                self.diagnostics.push(Diagnostic::error(
                                        format!("arithmetic operator '{}' cannot be applied to types {} and {}", op, l, r),
                                        Some(*span),
                                    ));
                            }
                            None
                        }
                        _ => None,
                    },

                    BinaryOp::Equal | BinaryOp::NotEqual => Some(TypeAnnotation::Bool),

                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => {
                        match (&left_type, &right_type) {
                            (Some(TypeAnnotation::Int), Some(TypeAnnotation::Int))
                            | (Some(TypeAnnotation::Float), Some(TypeAnnotation::Float)) => {
                                Some(TypeAnnotation::Bool)
                            }
                            (Some(l), Some(r)) => {
                                if *l != TypeAnnotation::Inferred
                                    && *r != TypeAnnotation::Inferred
                                    && l != r
                                {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!("comparison operator '{}' cannot compare types {} and {}", op, l, r),
                                        Some(*span),
                                    ));
                                }
                                Some(TypeAnnotation::Bool)
                            }
                            _ => Some(TypeAnnotation::Bool),
                        }
                    }

                    BinaryOp::And | BinaryOp::Or => {
                        if let Some(ref l) = left_type {
                            if *l != TypeAnnotation::Bool && *l != TypeAnnotation::Inferred {
                                self.diagnostics.push(Diagnostic::error(
                                    format!(
                                        "logical operator '{}' expects Bool left operand, found {}",
                                        op, l
                                    ),
                                    Some(left.span()),
                                ));
                            }
                        }
                        if let Some(ref r) = right_type {
                            if *r != TypeAnnotation::Bool && *r != TypeAnnotation::Inferred {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("logical operator '{}' expects Bool right operand, found {}", op, r),
                                    Some(right.span()),
                                ));
                            }
                        }
                        Some(TypeAnnotation::Bool)
                    }
                }
            }

            Expr::Grouping(inner, _) => self.analyze_expr(inner),
        }
    }

    fn types_compatible(&self, expected: &TypeAnnotation, actual: &TypeAnnotation) -> bool {
        if expected == &TypeAnnotation::Inferred || actual == &TypeAnnotation::Inferred {
            return true;
        }
        expected == actual
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_semantic_val_reassignment_error() {
        let code = "val x = 10\nx = 20";
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let result = SemanticAnalyzer::new().analyze(&program);

        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(
            errs[0]
                .message
                .contains("cannot reassign immutable variable `x`")
                || errs[0]
                    .message
                    .contains("cannot reassign immutable variable 'x'")
        );
    }

    #[test]
    fn test_semantic_var_reassignment_ok() {
        let code = "var x = 10\nx = 20";
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let result = SemanticAnalyzer::new().analyze(&program);

        assert!(result.is_ok());
    }

    #[test]
    fn test_semantic_undefined_variable() {
        let code = "println(undefined_var)";
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let result = SemanticAnalyzer::new().analyze(&program);

        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0]
            .message
            .contains("cannot find variable 'undefined_var'"));
    }
}
