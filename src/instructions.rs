
pub const SET: usize = 0x01;
pub const ADD: usize = 0x02;
pub const SUB: usize = 0x03;
pub const MUL: usize = 0x04;
pub const MLI: usize = 0x05;
pub const DIV: usize = 0x06;
pub const DVI: usize = 0x07;
pub const MOD: usize = 0x08;
pub const MDI: usize = 0x09;
pub const AND: usize = 0x0a;
pub const BOR: usize = 0x0b;
pub const XOR: usize = 0x0c;
pub const SHR: usize = 0x0d;
pub const ASR: usize = 0x0e;
pub const SHL: usize = 0x0f;
pub const IFB: usize = 0x10;
pub const IFC: usize = 0x11;
pub const IFE: usize = 0x12;
pub const IFN: usize = 0x13;
pub const IFG: usize = 0x14;
pub const IFA: usize = 0x15;
pub const IFL: usize = 0x16;
pub const IFU: usize = 0x17;
pub const ADX: usize = 0x1a;
pub const SBX: usize = 0x1b;
pub const STI: usize = 0x1e;
pub const STD: usize = 0x1f;

pub const JSR: usize = 0x01;
pub const INT: usize = 0x08;
pub const IAG: usize = 0x09;
pub const IAS: usize = 0x0a;
pub const RFI: usize = 0x0b;
pub const IAQ: usize = 0x0c;
pub const HWN: usize = 0x10;
pub const HWQ: usize = 0x11;
pub const HWI: usize = 0x12;

// Extended
pub const OUT: usize = 0x13;
