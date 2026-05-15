#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemTy {
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Unit,
    Never,
    Infer,
    Fn(Box<FnTy>),
    Struct(StructTy),
    Enum(EnumTy),
    Ref(Box<RefTy>),
    Tuple(Vec<SemTy>),
    Array(Box<SemTy>, usize),
    Slice(Box<SemTy>),
    Generic(String),
}

impl SemTy {
    pub fn from_ident(name: &str) -> Option<SemTy> {
        match name {
            "i32" => Some(SemTy::I32),
            "i64" => Some(SemTy::I64),
            "u32" => Some(SemTy::U32),
            "u64" => Some(SemTy::U64),
            "f32" => Some(SemTy::F32),
            "f64" => Some(SemTy::F64),
            "bool" => Some(SemTy::Bool),
            "char" => Some(SemTy::Char),
            "str" => Some(SemTy::Str),
            "()" | "Unit" => Some(SemTy::Unit),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SemTy::I32 => "i32",
            SemTy::I64 => "i64",
            SemTy::U32 => "u32",
            SemTy::U64 => "u64",
            SemTy::F32 => "f32",
            SemTy::F64 => "f64",
            SemTy::Bool => "bool",
            SemTy::Char => "char",
            SemTy::Str => "str",
            SemTy::Unit => "()",
            SemTy::Never => "!",
            SemTy::Infer => "_",
            SemTy::Fn(_) => "fn",
            SemTy::Struct(_) => "struct",
            SemTy::Enum(_) => "enum",
            SemTy::Ref(_) => "reference",
            SemTy::Tuple(_) => "tuple",
            SemTy::Array(_, _) => "array",
            SemTy::Slice(_) => "slice",
            SemTy::Generic(_) => "generic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnTy {
    pub params: Vec<SemTy>,
    pub ret: Box<SemTy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTy {
    pub name: String,
    pub fields: Vec<(String, SemTy)>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumTy {
    pub name: String,
    pub variants: Vec<(String, Option<Vec<SemTy>>)>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefTy {
    pub inner: Box<SemTy>,
    pub is_mut: bool,
}
