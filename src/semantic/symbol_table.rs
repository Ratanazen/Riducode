use crate::ast::TypeAnnotation;
use crate::diagnostics::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub is_mutable: bool,
    pub declared_type: Option<TypeAnnotation>,
    pub inferred_type: Option<TypeAnnotation>,
    pub span: Span,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn depth(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn define(&mut self, symbol: Symbol) -> Result<(), Symbol> {
        let current_scope = self
            .scopes
            .last_mut()
            .expect("at least one scope must exist");
        if let Some(existing) = current_scope.get(&symbol.name) {
            Err(existing.clone())
        } else {
            current_scope.insert(symbol.name.clone(), symbol);
            Ok(())
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn resolve_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(name) {
                return Some(sym);
            }
        }
        None
    }
}
