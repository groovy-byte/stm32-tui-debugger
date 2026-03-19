pub mod fetcher;
pub mod parser;
pub mod registry;

pub use fetcher::SvdFetcher;
pub use parser::SvdDevice;
pub use registry::{AccessType, BitField, PeripheralRegistry, Register, RegisterValue};
