#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub address: u32,
    pub size: usize,
    pub type_name: String,
    pub is_global: bool,
}

#[derive(Clone, Debug)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct TypeInfo {
    pub name: String,
    pub size: usize,
    pub kind: TypeKind,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Struct { members: Vec<StructMember> },
    Array { element_type: Box<TypeInfo>, length: usize },
    Pointer { target_type: Box<TypeInfo>, pointer_size: usize },
    Enum { variants: Vec<(String, i64)> },
    Typedef { underlying: Box<TypeInfo> },
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    Void,
}

#[derive(Clone, Debug)]
pub struct StructMember {
    pub name: String,
    pub offset: usize,
    pub type_info: TypeInfo,
}

#[derive(Clone, Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub source: Option<SourceLocation>,
}
