use crate::ast::{BinaryOp, Expr, Literal, Program, Stmt, TypeAnnotation, UnaryOp};
use crate::diagnostics::{Diagnostic, Span};
use crate::lexer::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Program, Vec<Diagnostic>> {
        let mut statements = Vec::new();
        let mut diagnostics = Vec::new();

        while !self.is_at_end() {
            // Skip redundant semicolons
            while self.match_token(&[TokenType::Semicolon]).is_some() {}

            if self.is_at_end() {
                break;
            }

            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(diag) => {
                    diagnostics.push(diag);
                    self.synchronize();
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(Program { statements })
        } else {
            Err(diagnostics)
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> Option<&Token> {
        for t in types {
            if self.check(t) {
                return Some(self.advance());
            }
        }
        None
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, Diagnostic> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let peek_token = self.peek();
            Err(Diagnostic::error(
                format!("expected {}, found {}", message, peek_token.token_type),
                Some(peek_token.span),
            ))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<(String, Span), Diagnostic> {
        if let TokenType::Identifier(ref id) = self.peek().token_type {
            let id_clone = id.clone();
            let span = self.advance().span;
            Ok((id_clone, span))
        } else {
            let peek_token = self.peek();
            Err(Diagnostic::error(
                format!("expected {}, found {}", message, peek_token.token_type),
                Some(peek_token.span),
            ))
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Val
                | TokenType::Var
                | TokenType::Fun
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Println
                | TokenType::Return
                | TokenType::RightBrace => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Diagnostic> {
        if self.match_token(&[TokenType::Val]).is_some() {
            self.var_declaration(false)
        } else if self.match_token(&[TokenType::Var]).is_some() {
            self.var_declaration(true)
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self, is_mutable: bool) -> Result<Stmt, Diagnostic> {
        let decl_keyword = self.previous().clone();
        let (name, _name_span) = self.consume_identifier("variable name after 'val' or 'var'")?;

        let type_annotation = if self.match_token(&[TokenType::Colon]).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&TokenType::Equal, "'=' in variable declaration")?;
        let initializer = self.expression()?;

        // Optional semicolon
        self.match_token(&[TokenType::Semicolon]);

        let span = Span::new(decl_keyword.span.start, initializer.span().end);
        Ok(Stmt::VarDecl {
            name,
            is_mutable,
            type_annotation,
            initializer,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<TypeAnnotation, Diagnostic> {
        let (id, span) = self.consume_identifier("type identifier")?;
        match id.as_str() {
            "Int" => Ok(TypeAnnotation::Int),
            "Float" => Ok(TypeAnnotation::Float),
            "String" => Ok(TypeAnnotation::String),
            "Bool" => Ok(TypeAnnotation::Bool),
            "Unit" => Ok(TypeAnnotation::Unit),
            _ => Err(Diagnostic::error(
                format!("unknown type '{}'", id),
                Some(span),
            )),
        }
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.match_token(&[TokenType::Println]).is_some() {
            self.print_statement(true)
        } else if self.match_token(&[TokenType::Print]).is_some() {
            self.print_statement(false)
        } else if self.check(&TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self, is_newline: bool) -> Result<Stmt, Diagnostic> {
        let keyword_span = self.previous().span;
        let has_paren = self.match_token(&[TokenType::LeftParen]).is_some();
        let expr = self.expression()?;

        if has_paren {
            self.consume(&TokenType::RightParen, "')' after print argument")?;
        }

        self.match_token(&[TokenType::Semicolon]);
        let span = Span::new(keyword_span.start, expr.span().end);

        if is_newline {
            Ok(Stmt::Println(expr, span))
        } else {
            Ok(Stmt::Print(expr, span))
        }
    }

    fn block(&mut self) -> Result<Stmt, Diagnostic> {
        let open_brace = self
            .consume(&TokenType::LeftBrace, "'{' to begin block")?
            .clone();
        let mut stmts = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            while self.match_token(&[TokenType::Semicolon]).is_some() {}
            if self.check(&TokenType::RightBrace) || self.is_at_end() {
                break;
            }
            stmts.push(self.declaration()?);
        }

        let close_brace = self.consume(&TokenType::RightBrace, "'}' after block")?;
        let span = Span::new(open_brace.span.start, close_brace.span.end);
        Ok(Stmt::Block(stmts, span))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let expr = self.expression()?;
        self.match_token(&[TokenType::Semicolon]);
        let span = expr.span();
        Ok(Stmt::Expression(expr, span))
    }

    pub fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.logical_or()?;

        if self.match_token(&[TokenType::Equal]).is_some() {
            let equals_token = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name, var_span) = expr {
                let span = Span::new(var_span.start, value.span().end);
                return Ok(Expr::Assign(name, Box::new(value), span));
            }

            return Err(Diagnostic::error(
                "invalid assignment target",
                Some(equals_token.span),
            ));
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.logical_and()?;

        while self.match_token(&[TokenType::OrOr]).is_some() {
            let right = self.logical_and()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right), span);
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::AndAnd]).is_some() {
            let right = self.equality()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right), span);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.comparison()?;

        while let Some(op_token) = self.match_token(&[TokenType::EqualEqual, TokenType::BangEqual])
        {
            let op = match op_token.token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.term()?;

        while let Some(op_token) = self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = match op_token.token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.factor()?;

        while let Some(op_token) = self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match op_token.token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.unary()?;

        while let Some(op_token) =
            self.match_token(&[TokenType::Star, TokenType::Slash, TokenType::Percent])
        {
            let op = match op_token.token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            let span = Span::new(expr.span().start, right.span().end);
            expr = Expr::Binary(Box::new(expr), op, Box::new(right), span);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        if let Some(op_token) = self.match_token(&[TokenType::Minus, TokenType::Bang]) {
            let op = match op_token.token_type {
                TokenType::Minus => UnaryOp::Negate,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            let start = op_token.span;
            let right = self.unary()?;
            let span = Span::new(start.start, right.span().end);
            return Ok(Expr::Unary(op, Box::new(right), span));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.peek().clone();

        match &token.token_type {
            TokenType::IntLiteral(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Int(val), token.span))
            }
            TokenType::FloatLiteral(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Float(val), token.span))
            }
            TokenType::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(val), token.span))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true), token.span))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false), token.span))
            }
            TokenType::Nil => {
                self.advance();
                Ok(Expr::Literal(Literal::Nil, token.span))
            }
            TokenType::Identifier(id) => {
                let name = id.clone();
                self.advance();
                Ok(Expr::Variable(name, token.span))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                let close_token = self.consume(&TokenType::RightParen, "')' after expression")?;
                let span = Span::new(token.span.start, close_token.span.end);
                Ok(Expr::Grouping(Box::new(expr), span))
            }
            _ => Err(Diagnostic::error(
                format!("expected expression, found {}", token.token_type),
                Some(token.span),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_declarations_and_expressions() {
        let code = "val x = 10 + 20 * 3\nvar y = x > 50\nprintln(y)";
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();

        assert_eq!(program.statements.len(), 3);
        match &program.statements[0] {
            Stmt::VarDecl {
                name, is_mutable, ..
            } => {
                assert_eq!(name, "x");
                assert!(!is_mutable);
            }
            _ => panic!("expected var decl"),
        }
    }
}
