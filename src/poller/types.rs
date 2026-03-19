use std::fmt;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Expression {
    pub id: String,
    pub name: String,
    pub address: Option<u32>,
    pub size: Option<usize>,
    pub format: ValueFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueFormat {
    Decimal,
    Hex,
    Binary,
    Float,
    Auto,
}

impl Default for ValueFormat {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Debug)]
pub struct PollResult {
    pub expr_id: String,
    pub value: DisplayValue,
    pub changed: bool,
    pub timestamp: Instant,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayValue {
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Struct(Vec<(String, DisplayValue)>),
    Array(Vec<DisplayValue>),
    Error(String),
}

impl fmt::Display for DisplayValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayValue::Integer(v) => write!(f, "{v}"),
            DisplayValue::Unsigned(v) => write!(f, "{v}"),
            DisplayValue::Float(v) => write!(f, "{v:.6}"),
            DisplayValue::Bytes(bytes) => {
                let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
                write!(f, "[{}]", hex.join(" "))
            }
            DisplayValue::Text(s) => write!(f, "{s}"),
            DisplayValue::Struct(members) => {
                write!(f, "{{ ")?;
                for (i, (name, val)) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {val}")?;
                }
                write!(f, " }}")
            }
            DisplayValue::Array(elems) => {
                write!(f, "[")?;
                for (i, val) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
            DisplayValue::Error(e) => write!(f, "<error: {e}>"),
        }
    }
}
