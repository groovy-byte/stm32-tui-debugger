use std::collections::HashMap;

use crate::error::SvdError;
use super::parser::{PeripheralInfo, SvdDevice};

#[derive(Clone, Debug)]
pub struct Register {
    pub name: String,
    pub address: u32,
    pub width: u32,
    pub fields: Vec<BitField>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BitField {
    pub name: String,
    pub offset: u32,
    pub width: u32,
    pub access: AccessType,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Clone, Debug)]
pub struct RegisterValue {
    pub register: Register,
    pub raw_value: u32,
    pub field_values: Vec<(String, u32)>,
}

pub struct PeripheralRegistry {
    peripherals: HashMap<String, PeripheralInfo>,
}

impl PeripheralRegistry {
    pub fn from_device(device: SvdDevice) -> Self {
        let peripherals = device
            .peripherals
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        PeripheralRegistry { peripherals }
    }

    pub fn list_peripherals(&self) -> Vec<String> {
        let mut names: Vec<_> = self.peripherals.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_peripheral(&self, name: &str) -> Result<&PeripheralInfo, SvdError> {
        self.peripherals
            .get(name)
            .ok_or_else(|| SvdError::PeripheralNotFound(name.to_string()))
    }

    pub fn get_registers(&self, peripheral: &str) -> Result<&[Register], SvdError> {
        let p = self.get_peripheral(peripheral)?;
        Ok(&p.registers)
    }

    pub fn get_register(&self, peripheral: &str, register: &str) -> Result<&Register, SvdError> {
        let p = self.get_peripheral(peripheral)?;
        p.registers
            .iter()
            .find(|r| r.name == register)
            .ok_or_else(|| SvdError::RegisterNotFound(format!("{}.{}", peripheral, register)))
    }

    pub fn decode_register(register: &Register, raw_value: u32) -> RegisterValue {
        let field_values = register
            .fields
            .iter()
            .map(|f| {
                let mask = if f.width >= 32 {
                    u32::MAX
                } else {
                    (1u32 << f.width) - 1
                };
                let value = (raw_value >> f.offset) & mask;
                (f.name.clone(), value)
            })
            .collect();

        RegisterValue {
            register: register.clone(),
            raw_value,
            field_values,
        }
    }
}
