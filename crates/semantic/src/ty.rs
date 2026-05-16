#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    ISize,
    USize,
    F32,
    F64,
    Bool,
    Char,
    Str,
    String,
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
            "i8" => Some(SemTy::I8),
            "i16" => Some(SemTy::I16),
            "i32" => Some(SemTy::I32),
            "i64" => Some(SemTy::I64),
            "i128" => Some(SemTy::I128),
            "u8" => Some(SemTy::U8),
            "u16" => Some(SemTy::U16),
            "u32" => Some(SemTy::U32),
            "u64" => Some(SemTy::U64),
            "u128" => Some(SemTy::U128),
            "isize" => Some(SemTy::ISize),
            "usize" => Some(SemTy::USize),
            "f32" => Some(SemTy::F32),
            "f64" => Some(SemTy::F64),
            "bool" => Some(SemTy::Bool),
            "char" => Some(SemTy::Char),
            "str" => Some(SemTy::Str),
            "String" => Some(SemTy::String),
            "()" | "Unit" => Some(SemTy::Unit),
            _ => None,
        }
    }

    pub fn from_suffix(suffix: &str) -> Option<SemTy> {
        match suffix {
            "i8" => Some(SemTy::I8),
            "i16" => Some(SemTy::I16),
            "i32" => Some(SemTy::I32),
            "i64" => Some(SemTy::I64),
            "i128" => Some(SemTy::I128),
            "u8" => Some(SemTy::U8),
            "u16" => Some(SemTy::U16),
            "u32" => Some(SemTy::U32),
            "u64" => Some(SemTy::U64),
            "u128" => Some(SemTy::U128),
            "isize" => Some(SemTy::ISize),
            "usize" => Some(SemTy::USize),
            "f32" => Some(SemTy::F32),
            "f64" => Some(SemTy::F64),
            _ => None,
        }
    }

    pub fn default_int() -> Self { SemTy::I32 }
    pub fn default_float() -> Self { SemTy::F64 }

    pub fn is_integer(&self) -> bool {
        matches!(self, SemTy::I8 | SemTy::I16 | SemTy::I32 | SemTy::I64 | SemTy::I128
            | SemTy::U8 | SemTy::U16 | SemTy::U32 | SemTy::U64 | SemTy::U128
            | SemTy::ISize | SemTy::USize)
    }

    pub fn name(&self) -> &'static str {
        match self {
            SemTy::I8 => "i8",
            SemTy::I16 => "i16",
            SemTy::I32 => "i32",
            SemTy::I64 => "i64",
            SemTy::I128 => "i128",
            SemTy::U8 => "u8",
            SemTy::U16 => "u16",
            SemTy::U32 => "u32",
            SemTy::U64 => "u64",
            SemTy::U128 => "u128",
            SemTy::ISize => "isize",
            SemTy::USize => "usize",
            SemTy::F32 => "f32",
            SemTy::F64 => "f64",
            SemTy::Bool => "bool",
            SemTy::Char => "char",
            SemTy::Str => "str",
            SemTy::String => "String",
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
