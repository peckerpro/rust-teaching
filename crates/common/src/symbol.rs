use std::{
    collections::HashMap,
    fmt,
    sync::RwLock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

impl Symbol {
    pub const ROOT: Symbol = Symbol(0);

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

pub struct Interner {
    sym_to_string: RwLock<Vec<String>>,
    string_to_sym: RwLock<HashMap<String, Symbol>>,
}

impl Interner {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(String::new(), Symbol::ROOT);
        Interner {
            sym_to_string: RwLock::new(vec![String::new()]),
            string_to_sym: RwLock::new(map),
        }
    }

    pub fn intern(&self, s: &str) -> Symbol {
        {
            let map = self.string_to_sym.read().unwrap();
            if let Some(sym) = map.get(s) {
                return *sym;
            }
        }
        let mut strings = self.sym_to_string.write().unwrap();
        let idx = strings.len() as u32;
        strings.push(s.to_string());
        let sym = Symbol(idx);
        let mut map = self.string_to_sym.write().unwrap();
        map.insert(s.to_string(), sym);
        sym
    }

    pub fn resolve(&self, sym: Symbol) -> String {
        let strings = self.sym_to_string.read().unwrap();
        strings[sym.0 as usize].clone()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}
