use crate::span::{SourceFile, Span};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub span: Span,
    pub file: Arc<SourceFile>,
}

impl Diagnostic {
    pub fn new(level: Level, message: impl Into<String>, span: Span, file: Arc<SourceFile>) -> Self {
        Diagnostic {
            level,
            message: message.into(),
            span,
            file,
        }
    }
}

pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        DiagnosticBag {
            diagnostics: Vec::new(),
            error_count: 0,
        }
    }

    pub fn error(&mut self, message: impl Into<String>, span: Span, file: Arc<SourceFile>) {
        self.error_count += 1;
        self.diagnostics
            .push(Diagnostic::new(Level::Error, message, span, file));
    }

    pub fn warn(&mut self, message: impl Into<String>, span: Span, file: Arc<SourceFile>) {
        self.diagnostics
            .push(Diagnostic::new(Level::Warning, message, span, file));
    }

    pub fn note(&mut self, message: impl Into<String>, span: Span, file: Arc<SourceFile>) {
        self.diagnostics
            .push(Diagnostic::new(Level::Note, message, span, file));
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl Default for DiagnosticBag {
    fn default() -> Self {
        Self::new()
    }
}
