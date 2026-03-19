use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Probe error: {0}")]
    Probe(#[from] ProbeError),
    #[error("Symbol error: {0}")]
    Symbol(#[from] SymbolError),
    #[error("SVD error: {0}")]
    Svd(#[from] SvdError),
    #[error("Poller error: {0}")]
    Poller(#[from] PollerError),
    #[error("TUI error: {0}")]
    Tui(#[from] TuiError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("Failed to connect to probe: {0}")]
    ConnectionFailed(String),
    #[error("No active session")]
    NoSession,
    #[error("Target not halted")]
    TargetRunning,
    #[error("Memory read failed at 0x{addr:08x}: {reason}")]
    MemoryReadFailed { addr: u32, reason: String },
    #[error("Memory write failed at 0x{addr:08x}: {reason}")]
    MemoryWriteFailed { addr: u32, reason: String },
    #[error("Register access failed: {0}")]
    RegisterAccessFailed(String),
}

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("ELF load failed: {0}")]
    ElfLoadFailed(String),
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("Type resolution failed for: {0}")]
    TypeResolutionFailed(String),
    #[error("DWARF parsing error: {0}")]
    DwarfParse(String),
    #[error("Demangling failed: {0}")]
    DemangleFailed(String),
}

#[derive(Error, Debug)]
pub enum SvdError {
    #[error("SVD fetch failed: {0}")]
    FetchFailed(String),
    #[error("SVD parse error: {0}")]
    ParseError(String),
    #[error("Peripheral not found: {0}")]
    PeripheralNotFound(String),
    #[error("Register not found: {0}")]
    RegisterNotFound(String),
}

#[derive(Error, Debug)]
pub enum PollerError {
    #[error("Expression parse failed: {0}")]
    ExpressionParseFailed(String),
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
}

#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Terminal setup failed: {0}")]
    TerminalSetupFailed(String),
    #[error("Rendering error: {0}")]
    RenderFailed(String),
}
