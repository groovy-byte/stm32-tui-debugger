use stm32_tui_debugger::svd::{
    AccessType, BitField, PeripheralRegistry, Register, SvdDevice,
};

const MINIMAL_SVD: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<device>
  <name>TestDevice</name>
  <peripherals>
    <peripheral>
      <name>GPIOA</name>
      <baseAddress>0x40020000</baseAddress>
      <registers>
        <register>
          <name>MODER</name>
          <addressOffset>0x00</addressOffset>
          <size>32</size>
          <fields>
            <field>
              <name>MODER0</name>
              <bitOffset>0</bitOffset>
              <bitWidth>2</bitWidth>
            </field>
            <field>
              <name>MODER1</name>
              <bitOffset>2</bitOffset>
              <bitWidth>2</bitWidth>
            </field>
          </fields>
        </register>
        <register>
          <name>ODR</name>
          <addressOffset>0x14</addressOffset>
          <size>32</size>
          <fields>
            <field>
              <name>ODR0</name>
              <bitOffset>0</bitOffset>
              <bitWidth>1</bitWidth>
            </field>
          </fields>
        </register>
      </registers>
    </peripheral>
    <peripheral>
      <name>GPIOB</name>
      <baseAddress>0x40020400</baseAddress>
      <registers>
        <register>
          <name>MODER</name>
          <addressOffset>0x00</addressOffset>
          <size>32</size>
        </register>
      </registers>
    </peripheral>
  </peripherals>
</device>"#;

fn make_registry() -> PeripheralRegistry {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).expect("Failed to parse minimal SVD");
    PeripheralRegistry::from_device(device)
}

// ── SvdDevice::parse_xml ────────────────────────────────────────────────

#[test]
fn should_parse_device_name() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    assert_eq!(device.name, "TestDevice");
}

#[test]
fn should_parse_two_peripherals() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    assert_eq!(device.peripherals.len(), 2);
}

#[test]
fn should_parse_gpioa_base_address() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    assert_eq!(gpioa.base_address, 0x4002_0000);
}

#[test]
fn should_parse_register_count_for_gpioa() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    assert_eq!(gpioa.registers.len(), 2);
}

#[test]
fn should_parse_register_address() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    let moder = gpioa.registers.iter().find(|r| r.name == "MODER").unwrap();
    // base (0x40020000) + offset (0x00) = 0x40020000
    assert_eq!(moder.address, 0x4002_0000);
}

#[test]
fn should_parse_register_odr_address() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    let odr = gpioa.registers.iter().find(|r| r.name == "ODR").unwrap();
    // base (0x40020000) + offset (0x14) = 0x40020014
    assert_eq!(odr.address, 0x4002_0014);
}

#[test]
fn should_parse_bitfield_names() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    let moder = gpioa.registers.iter().find(|r| r.name == "MODER").unwrap();
    let field_names: Vec<&str> = moder.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(field_names.contains(&"MODER0"));
    assert!(field_names.contains(&"MODER1"));
}

#[test]
fn should_parse_bitfield_offsets_and_widths() {
    let device = SvdDevice::parse_xml(MINIMAL_SVD).unwrap();
    let gpioa = device
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap();
    let moder = gpioa.registers.iter().find(|r| r.name == "MODER").unwrap();
    let m0 = moder.fields.iter().find(|f| f.name == "MODER0").unwrap();
    assert_eq!(m0.offset, 0);
    assert_eq!(m0.width, 2);
    let m1 = moder.fields.iter().find(|f| f.name == "MODER1").unwrap();
    assert_eq!(m1.offset, 2);
    assert_eq!(m1.width, 2);
}

#[test]
fn should_reject_invalid_svd_xml() {
    let result = SvdDevice::parse_xml("<not-a-device/>");
    assert!(result.is_err());
}

// ── PeripheralRegistry ──────────────────────────────────────────────────

#[test]
fn should_list_peripherals_sorted() {
    let reg = make_registry();
    let names = reg.list_peripherals();
    assert_eq!(names, vec!["GPIOA", "GPIOB"]);
}

#[test]
fn should_get_existing_peripheral() {
    let reg = make_registry();
    let p = reg.get_peripheral("GPIOA");
    assert!(p.is_ok());
    assert_eq!(p.unwrap().name, "GPIOA");
}

#[test]
fn should_error_on_missing_peripheral() {
    let reg = make_registry();
    let p = reg.get_peripheral("SPI1");
    assert!(p.is_err());
}

#[test]
fn should_get_registers_for_peripheral() {
    let reg = make_registry();
    let regs = reg.get_registers("GPIOA").unwrap();
    assert_eq!(regs.len(), 2);
}

#[test]
fn should_get_specific_register() {
    let reg = make_registry();
    let r = reg.get_register("GPIOA", "MODER");
    assert!(r.is_ok());
    assert_eq!(r.unwrap().name, "MODER");
}

#[test]
fn should_error_on_missing_register() {
    let reg = make_registry();
    let r = reg.get_register("GPIOA", "NONEXISTENT");
    assert!(r.is_err());
}

#[test]
fn should_error_on_register_of_missing_peripheral() {
    let reg = make_registry();
    let r = reg.get_register("SPI1", "CR1");
    assert!(r.is_err());
}

// ── decode_register ─────────────────────────────────────────────────────

#[test]
fn should_decode_register_all_zeros() {
    let register = Register {
        name: "MODER".into(),
        address: 0x4002_0000,
        width: 32,
        fields: vec![
            BitField {
                name: "MODER0".into(),
                offset: 0,
                width: 2,
                access: AccessType::ReadWrite,
                description: None,
            },
            BitField {
                name: "MODER1".into(),
                offset: 2,
                width: 2,
                access: AccessType::ReadWrite,
                description: None,
            },
        ],
        description: None,
    };

    let decoded = PeripheralRegistry::decode_register(&register, 0x0000_0000);
    assert_eq!(decoded.raw_value, 0);
    assert_eq!(decoded.field_values.len(), 2);
    assert_eq!(decoded.field_values[0], ("MODER0".into(), 0));
    assert_eq!(decoded.field_values[1], ("MODER1".into(), 0));
}

#[test]
fn should_decode_register_with_bitfield_values() {
    let register = Register {
        name: "MODER".into(),
        address: 0x4002_0000,
        width: 32,
        fields: vec![
            BitField {
                name: "MODER0".into(),
                offset: 0,
                width: 2,
                access: AccessType::ReadWrite,
                description: None,
            },
            BitField {
                name: "MODER1".into(),
                offset: 2,
                width: 2,
                access: AccessType::ReadWrite,
                description: None,
            },
        ],
        description: None,
    };

    // raw_value = 0b1001 → MODER0 = 0b01 (1), MODER1 = 0b10 (2)
    let decoded = PeripheralRegistry::decode_register(&register, 0b1001);
    assert_eq!(decoded.field_values[0], ("MODER0".into(), 1));
    assert_eq!(decoded.field_values[1], ("MODER1".into(), 2));
}

#[test]
fn should_decode_register_all_ones() {
    let register = Register {
        name: "MODER".into(),
        address: 0x4002_0000,
        width: 32,
        fields: vec![BitField {
            name: "MODER0".into(),
            offset: 0,
            width: 2,
            access: AccessType::ReadWrite,
            description: None,
        }],
        description: None,
    };

    let decoded = PeripheralRegistry::decode_register(&register, 0xFFFF_FFFF);
    assert_eq!(decoded.field_values[0], ("MODER0".into(), 3)); // 2 bits → max 3
}

#[test]
fn should_decode_single_bit_field() {
    let register = Register {
        name: "ODR".into(),
        address: 0x4002_0014,
        width: 32,
        fields: vec![BitField {
            name: "ODR0".into(),
            offset: 0,
            width: 1,
            access: AccessType::ReadWrite,
            description: None,
        }],
        description: None,
    };

    let decoded = PeripheralRegistry::decode_register(&register, 0b1);
    assert_eq!(decoded.field_values[0], ("ODR0".into(), 1));
}

#[test]
fn should_decode_high_offset_field() {
    let register = Register {
        name: "TEST".into(),
        address: 0x4002_0000,
        width: 32,
        fields: vec![BitField {
            name: "HIGH_BITS".into(),
            offset: 28,
            width: 4,
            access: AccessType::ReadWrite,
            description: None,
        }],
        description: None,
    };

    // 0xA000_0000 → bits 31:28 = 0xA = 10
    let decoded = PeripheralRegistry::decode_register(&register, 0xA000_0000);
    assert_eq!(decoded.field_values[0], ("HIGH_BITS".into(), 0xA));
}

// ── SvdFetcher (chip_to_svd_filename via is_cached) ─────────────────────

#[test]
fn should_map_chip_name_to_svd_filename_via_fetcher() {
    use stm32_tui_debugger::svd::SvdFetcher;
    use std::path::PathBuf;

    let tmpdir = std::env::temp_dir().join("stm32_tui_test_svd_fetcher");
    let _ = std::fs::create_dir_all(&tmpdir);

    // Write a fake SVD file named stm32f407.svd
    std::fs::write(tmpdir.join("stm32f407.svd"), "<device/>").ok();

    let fetcher = SvdFetcher::new(tmpdir.clone());
    // If is_cached returns true, it means chip_to_svd_filename mapped correctly
    assert!(fetcher.is_cached("STM32F407VG"));

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmpdir);
}

#[test]
fn should_not_find_uncached_chip() {
    use stm32_tui_debugger::svd::SvdFetcher;

    let tmpdir = std::env::temp_dir().join("stm32_tui_test_svd_fetcher_empty");
    let _ = std::fs::create_dir_all(&tmpdir);

    let fetcher = SvdFetcher::new(tmpdir.clone());
    assert!(!fetcher.is_cached("STM32H563ZI"));

    let _ = std::fs::remove_dir_all(&tmpdir);
}
