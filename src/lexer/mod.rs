pub mod token;

use crate::diagnostics::{Diagnostic, SourceLocation, Span};
use token::{Token, TokenType};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            cursor: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
        let mut tokens = Vec::new();
        let mut diagnostics = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments(&mut diagnostics);
            if self.is_at_end() {
                break;
            }

            match self.next_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(diag) => diagnostics.push(diag),
            }
        }

        let eof_loc = self.current_location();
        tokens.push(Token::new(TokenType::Eof, Span::point(eof_loc), ""));

        if diagnostics.is_empty() {
            Ok(tokens)
        } else {
            Err(diagnostics)
        }
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.chars.len()
    }

    fn current_location(&self) -> SourceLocation {
        let offset = if self.cursor < self.chars.len() {
            self.chars[self.cursor].0
        } else {
            self.source.len()
        };
        SourceLocation::new(self.line, self.column, offset)
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.chars[self.cursor].1)
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.cursor + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.cursor + 1].1)
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            let (_, ch) = self.chars[self.cursor];
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace_and_comments(&mut self, diagnostics: &mut Vec<Diagnostic>) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        self.advance();
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        let start_loc = self.current_location();
                        self.advance();
                        self.advance();
                        let mut closed = false;
                        while !self.is_at_end() {
                            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                closed = true;
                                break;
                            }
                            self.advance();
                        }
                        if !closed {
                            let end_loc = self.current_location();
                            diagnostics.push(Diagnostic::error(
                                "unterminated block comment",
                                Some(Span::new(start_loc, end_loc)),
                            ));
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, Diagnostic> {
        let start_loc = self.current_location();
        let ch = match self.advance() {
            Some(c) => c,
            None => return Ok(None),
        };

        let token_type = match ch {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ':' => TokenType::Colon,
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Percent,

            '=' => {
                if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenType::AndAnd
                } else {
                    let end_loc = self.current_location();
                    return Err(Diagnostic::error(
                        "unexpected character '&', did you mean '&&'?",
                        Some(Span::new(start_loc, end_loc)),
                    ));
                }
            }
            '|' => {
                if self.match_char('|') {
                    TokenType::OrOr
                } else {
                    let end_loc = self.current_location();
                    return Err(Diagnostic::error(
                        "unexpected character '|', did you mean '||'?",
                        Some(Span::new(start_loc, end_loc)),
                    ));
                }
            }

            '"' => return self.scan_string(start_loc),

            c if c.is_ascii_digit() => return self.scan_number(start_loc, c),

            c if c.is_alphabetic() || c == '_' => return self.scan_identifier(start_loc, c),

            unknown => {
                let end_loc = self.current_location();
                return Err(Diagnostic::error(
                    format!("unexpected character '{}'", unknown),
                    Some(Span::new(start_loc, end_loc)),
                ));
            }
        };

        let end_loc = self.current_location();
        let lexeme = self.extract_lexeme(start_loc.offset, end_loc.offset);
        Ok(Some(Token::new(
            token_type,
            Span::new(start_loc, end_loc),
            lexeme,
        )))
    }

    fn scan_string(&mut self, start_loc: SourceLocation) -> Result<Option<Token>, Diagnostic> {
        let mut content = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                let end_loc = self.current_location();
                let lexeme = self.extract_lexeme(start_loc.offset, end_loc.offset);
                return Ok(Some(Token::new(
                    TokenType::StringLiteral(content),
                    Span::new(start_loc, end_loc),
                    lexeme,
                )));
            }

            if c == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => content.push('\n'),
                    Some('t') => content.push('\t'),
                    Some('r') => content.push('\r'),
                    Some('\\') => content.push('\\'),
                    Some('"') => content.push('"'),
                    Some('0') => content.push('\0'),
                    Some(other) => {
                        let err_loc = self.current_location();
                        return Err(Diagnostic::error(
                            format!("unknown escape sequence '\\{}'", other),
                            Some(Span::point(err_loc)),
                        ));
                    }
                    None => {
                        let end_loc = self.current_location();
                        return Err(Diagnostic::error(
                            "unterminated string literal escape",
                            Some(Span::new(start_loc, end_loc)),
                        ));
                    }
                }
            } else {
                content.push(c);
                self.advance();
            }
        }

        let end_loc = self.current_location();
        Err(Diagnostic::error(
            "unterminated string literal",
            Some(Span::new(start_loc, end_loc)),
        ))
    }

    fn scan_number(
        &mut self,
        start_loc: SourceLocation,
        first_digit: char,
    ) -> Result<Option<Token>, Diagnostic> {
        let mut num_str = String::new();
        num_str.push(first_digit);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    num_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        let is_float = if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            num_str.push('.');
            self.advance(); // consume '.'
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        num_str.push(c);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
            true
        } else {
            false
        };

        let end_loc = self.current_location();
        let lexeme = self.extract_lexeme(start_loc.offset, end_loc.offset);

        if is_float {
            match num_str.parse::<f64>() {
                Ok(val) => Ok(Some(Token::new(
                    TokenType::FloatLiteral(val),
                    Span::new(start_loc, end_loc),
                    lexeme,
                ))),
                Err(e) => Err(Diagnostic::error(
                    format!("invalid floating point literal '{}': {}", num_str, e),
                    Some(Span::new(start_loc, end_loc)),
                )),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(val) => Ok(Some(Token::new(
                    TokenType::IntLiteral(val),
                    Span::new(start_loc, end_loc),
                    lexeme,
                ))),
                Err(e) => Err(Diagnostic::error(
                    format!("invalid integer literal '{}': {}", num_str, e),
                    Some(Span::new(start_loc, end_loc)),
                )),
            }
        }
    }

    fn scan_identifier(
        &mut self,
        start_loc: SourceLocation,
        first_char: char,
    ) -> Result<Option<Token>, Diagnostic> {
        let mut ident = String::new();
        ident.push(first_char);

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let end_loc = self.current_location();
        let lexeme = self.extract_lexeme(start_loc.offset, end_loc.offset);

        let token_type = match ident.as_str() {
            "val" => TokenType::Val,
            "var" => TokenType::Var,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            "print" => TokenType::Print,
            "println" => TokenType::Println,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nil" => TokenType::Nil,
            _ => TokenType::Identifier(ident),
        };

        Ok(Some(Token::new(
            token_type,
            Span::new(start_loc, end_loc),
            lexeme,
        )))
    }

    fn extract_lexeme(&self, start_offset: usize, end_offset: usize) -> String {
        if start_offset <= end_offset && end_offset <= self.source.len() {
            self.source[start_offset..end_offset].to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_basic_tokens() {
        let code = "val x = 42\nvar y = 3.14\nprintln(\"Hello, Ridu!\")";
        let tokens = Lexer::new(code)
            .tokenize()
            .expect("tokenization should succeed");

        assert_eq!(tokens[0].token_type, TokenType::Val);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".into()));
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::IntLiteral(42));
        assert_eq!(tokens[4].token_type, TokenType::Var);
        assert_eq!(tokens[5].token_type, TokenType::Identifier("y".into()));
        assert_eq!(tokens[6].token_type, TokenType::Equal);
        assert_eq!(tokens[7].token_type, TokenType::FloatLiteral(3.14));
        assert_eq!(tokens[8].token_type, TokenType::Println);
        assert_eq!(tokens[9].token_type, TokenType::LeftParen);
        assert_eq!(
            tokens[10].token_type,
            TokenType::StringLiteral("Hello, Ridu!".into())
        );
        assert_eq!(tokens[11].token_type, TokenType::RightParen);
        assert_eq!(tokens[12].token_type, TokenType::Eof);
    }

    #[test]
    fn test_comments_and_whitespace() {
        let code = "// line comment\nval a = 1 /* block */ + 2";
        let tokens = Lexer::new(code)
            .tokenize()
            .expect("tokenization should succeed");
        assert_eq!(tokens[0].token_type, TokenType::Val);
        assert_eq!(tokens[3].token_type, TokenType::IntLiteral(1));
        assert_eq!(tokens[4].token_type, TokenType::Plus);
        assert_eq!(tokens[5].token_type, TokenType::IntLiteral(2));
    }
}
