use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Span {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn point(loc: SourceLocation) -> Self {
        Self {
            start: loc,
            end: loc,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
    pub file_path: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            span,
            file_path: None,
        }
    }

    pub fn warning(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            span,
            file_path: None,
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file_path = Some(file.into());
        self
    }

    pub fn render(&self, source: Option<&str>) -> String {
        let file_display = self.file_path.as_deref().unwrap_or("<input>");
        let mut out = String::new();

        if let Some(span) = self.span {
            out.push_str(&format!(
                "{}:{}:{}: {}: {}\n",
                file_display, span.start.line, span.start.column, self.level, self.message
            ));

            if let Some(src) = source {
                if let Some(line_content) = src.lines().nth(span.start.line.saturating_sub(1)) {
                    out.push_str(&format!("{:5} | {}\n", span.start.line, line_content));
                    let indent = " ".repeat(span.start.column.saturating_sub(1));
                    let caret_len = if span.end.line == span.start.line
                        && span.end.column > span.start.column
                    {
                        span.end.column - span.start.column
                    } else {
                        1
                    };
                    let carets = "^".repeat(caret_len.max(1));
                    out.push_str(&format!("      | {}{}\n", indent, carets));
                }
            }
        } else {
            out.push_str(&format!(
                "{}: {}: {}\n",
                file_display, self.level, self.message
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_rendering() {
        let src = "val x = 10\nx = 20";
        let diag = Diagnostic::error(
            "cannot reassign immutable variable `x`",
            Some(Span::new(
                SourceLocation::new(2, 1, 11),
                SourceLocation::new(2, 2, 12),
            )),
        )
        .with_file("test.ridu");

        let rendered = diag.render(Some(src));
        assert!(rendered.contains("test.ridu:2:1: error: cannot reassign immutable variable `x`"));
        assert!(rendered.contains("x = 20"));
        assert!(rendered.contains("^"));
    }
}
