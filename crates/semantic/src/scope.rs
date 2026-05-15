use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    symbols: HashMap<String, SymbolEntry>,
}

#[derive(Debug, Clone)]
pub enum SymbolEntry {
    Var(VarInfo),
    Fn(FnInfo),
    Struct(StructInfo),
    Enum(EnumInfo),
    TypeAlias(TypeAliasInfo),
    Trait(TraitInfo),
    Module(ModuleInfo),
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Option<crate::ty::SemTy>,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub struct FnInfo {
    pub params: Vec<crate::ty::SemTy>,
    pub ret: Option<crate::ty::SemTy>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: Vec<(String, crate::ty::SemTy)>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub variants: Vec<(String, Option<Vec<crate::ty::SemTy>>)>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TypeAliasInfo {
    pub ty: crate::ty::SemTy,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub methods: Vec<(String, Vec<crate::ty::SemTy>, Option<crate::ty::SemTy>)>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub scope: Box<Scope>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            parent: None,
            symbols: HashMap::new(),
        }
    }

    pub fn child(parent: Scope) -> Self {
        Scope {
            parent: Some(Box::new(parent)),
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, entry: SymbolEntry) {
        self.symbols.insert(name, entry);
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        if let Some(entry) = self.symbols.get(name) {
            return Some(entry);
        }
        self.parent.as_ref().and_then(|p| p.lookup(name))
    }

    pub fn lookup_local(&self, name: &str) -> Option<&SymbolEntry> {
        self.symbols.get(name)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, SymbolEntry> {
        self.symbols.iter()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
