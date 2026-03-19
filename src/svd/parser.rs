use std::path::Path;

use svd_parser::svd::{Access, RegisterCluster};

use super::registry::{AccessType, BitField, Register};
use crate::error::SvdError;

pub struct SvdDevice {
    pub name: String,
    pub peripherals: Vec<PeripheralInfo>,
}

#[derive(Clone, Debug)]
pub struct PeripheralInfo {
    pub name: String,
    pub base_address: u32,
    pub description: Option<String>,
    pub registers: Vec<Register>,
}

impl SvdDevice {
    pub fn parse_file(path: &Path) -> Result<Self, SvdError> {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| SvdError::ParseError(format!("Failed to read SVD file: {}", e)))?;
        Self::parse_xml(&xml)
    }

    pub fn parse_xml(xml: &str) -> Result<Self, SvdError> {
        let device =
            svd_parser::parse(xml).map_err(|e| SvdError::ParseError(e.to_string()))?;

        let peripherals = device
            .peripherals
            .iter()
            .map(|p| {
                let base = p.base_address as u32;
                let registers = collect_registers(p.registers.as_deref(), base);

                PeripheralInfo {
                    name: p.name.clone(),
                    base_address: base,
                    description: p.description.clone(),
                    registers,
                }
            })
            .collect();

        Ok(SvdDevice {
            name: device.name,
            peripherals,
        })
    }
}

/// Recursively collect registers from a list of RegisterCluster nodes,
/// accumulating the address offset from enclosing clusters.
fn collect_registers(rcs: Option<&[RegisterCluster]>, base_offset: u32) -> Vec<Register> {
    let Some(rcs) = rcs else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for rc in rcs {
        match rc {
            RegisterCluster::Register(r) => {
                let addr = base_offset + r.address_offset;
                let fields = r
                    .fields
                    .as_ref()
                    .map(|fields| {
                        fields
                            .iter()
                            .map(|f| BitField {
                                name: f.name.clone(),
                                offset: f.bit_range.offset,
                                width: f.bit_range.width,
                                access: convert_access(f.access),
                                description: f.description.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                result.push(Register {
                    name: r.name.clone(),
                    address: addr,
                    width: r.properties.size.unwrap_or(32),
                    fields,
                    description: r.description.clone(),
                });
            }
            RegisterCluster::Cluster(c) => {
                let cluster_base = base_offset + c.address_offset;
                result.extend(collect_registers(Some(&c.children), cluster_base));
            }
        }
    }
    result
}

fn convert_access(access: Option<Access>) -> AccessType {
    match access {
        Some(Access::ReadOnly) => AccessType::ReadOnly,
        Some(Access::WriteOnly) | Some(Access::WriteOnce) => AccessType::WriteOnly,
        _ => AccessType::ReadWrite,
    }
}
