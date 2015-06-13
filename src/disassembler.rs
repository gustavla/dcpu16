use dcpu::DCPU;
use instructions::*;

#[allow(dead_code)]
enum ShellColor {
    Default,
    Blue,
    Green,
    Cyan,
    Red,
    Purple,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightPurple,
    White,
    Yellow,
}

const COLOR_INSTRUCTION: ShellColor = ShellColor::White;
const COLOR_NUM_LITERAL: ShellColor = ShellColor::LightPurple;
const COLOR_REGISTRY: ShellColor = ShellColor::Yellow;
const COLOR_NAMED: ShellColor = ShellColor::LightCyan;

fn maybe_colorize(s: String, color: ShellColor, enable: bool) -> String {
    if enable {
        colorize(s, color)
    } else {
        s
    }
}

fn colorize(s: String, color: ShellColor) -> String {
    match color {
        ShellColor::Blue => format!("\x1b[0;34m{}\x1b[0m", s),
        ShellColor::Green => format!("\x1b[0;32m{}\x1b[0m", s),
        ShellColor::Cyan => format!("\x1b[0;36m{}\x1b[0m", s),
        ShellColor::Red => format!("\x1b[0;31m{}\x1b[0m", s),
        ShellColor::Purple => format!("\x1b[0;35m{}\x1b[0m", s),
        ShellColor::Brown => format!("\x1b[0;33m{}\x1b[0m", s),
        ShellColor::LightGray => format!("\x1b[0;37m{}\x1b[0m", s),

        ShellColor::DarkGray => format!("\x1b[1;30m{}\x1b[0m", s),
        ShellColor::LightBlue => format!("\x1b[1;34m{}\x1b[0m", s),
        ShellColor::LightGreen => format!("\x1b[1;32m{}\x1b[0m", s),
        ShellColor::LightCyan => format!("\x1b[1;36m{}\x1b[0m", s),
        ShellColor::LightRed => format!("\x1b[1;31m{}\x1b[0m", s),
        ShellColor::LightPurple => format!("\x1b[1;35m{}\x1b[0m", s),
        ShellColor::Yellow => format!("\x1b[1;33m{}\x1b[0m", s),
        ShellColor::White => format!("\x1b[1;37m{}\x1b[0m", s),

        _ => s,
    }
}

fn opcode_str(opcode: usize) -> Result<&'static str, ()> {
    match opcode {
        SET => Ok("SET"),
        ADD => Ok("ADD"),
        SUB => Ok("SUB"),
        MUL => Ok("MUL"),
        MLI => Ok("MLI"),
        DIV => Ok("DIV"),
        DVI => Ok("DVI"),
        MOD => Ok("MOD"),
        MDI => Ok("MDI"),
        AND => Ok("AND"),
        BOR => Ok("BOR"),
        XOR => Ok("XOR"),
        SHR => Ok("SHR"),
        ASR => Ok("ASR"),
        SHL => Ok("SHL"),
        IFB => Ok("IFB"),
        IFC => Ok("IFC"),
        IFE => Ok("IFE"),
        IFN => Ok("IFN"),
        IFG => Ok("IFG"),
        IFA => Ok("IFA"),
        IFL => Ok("IFL"),
        IFU => Ok("IFU"),
        ADX => Ok("ADX"),
        SBX => Ok("SBX"),
        STI => Ok("STI"),
        STD => Ok("STD"),
        _ => Err(()),
    }
}

fn special_opcode_str(opcode: usize) -> Result<&'static str, ()> {
    match opcode {
        JSR => Ok("JSR"),
        INT => Ok("INT"),
        IAG => Ok("IAG"),
        IAS => Ok("IAS"),
        RFI => Ok("RFI"),
        IAQ => Ok("IAQ"),
        HWN => Ok("HWN"),
        HWQ => Ok("HWQ"),
        HWI => Ok("HWI"),
        // Extra
        OUT => Ok("OUT"),
        _ => Err(()),
    }
}

fn value_str(cpu: &DCPU, value: usize, offset: &mut u16,
             lvalue: bool, use_color: bool) -> String {
    match value {
        0x00 ... 0x07  => { 
            let s = registry_str(value).to_string();
            maybe_colorize(s, COLOR_REGISTRY, use_color)
        },
        0x08 ... 0x0f => { 
            let s = registry_str(value-0x08).to_string();
            let ss = maybe_colorize(s, COLOR_REGISTRY, use_color);
            format!("[{}]", ss)
        },
        0x10 ... 0x17 => {
            let v = cpu.mem[(cpu.pc + *offset) as usize];
            let s = registry_str(value-0x10).to_string();
            let ss = maybe_colorize(s, COLOR_REGISTRY, use_color);
            let vv = maybe_colorize(format!("{}", v), COLOR_NUM_LITERAL, use_color);
            *offset += 1;
            format!("[{}+{}]", ss, vv)
        },
        0x18 => {
            if lvalue {
                maybe_colorize("PUSH".to_string(), COLOR_NAMED, use_color)
            } else {
                maybe_colorize("POP".to_string(), COLOR_NAMED, use_color)
            }
        },
        0x19 => { 
            maybe_colorize("PEEK".to_string(), COLOR_NAMED, use_color)
        },
        0x1a => {
            *offset += 1;
            // TODO n needs to be the actual value
            maybe_colorize("PICK n".to_string(), COLOR_NAMED, use_color)
        },
        0x1b => {
            maybe_colorize("SP".to_string(), COLOR_NAMED, use_color)
        },
        0x1c => { 
            maybe_colorize("PC".to_string(), COLOR_NAMED, use_color)
        },
        0x1d => { 
            maybe_colorize("EX".to_string(), COLOR_NAMED, use_color)
        },
        0x1e => {
            let v = cpu.mem[(cpu.pc + *offset) as usize];
            let ss = maybe_colorize(format!("0x{:04x}", v), COLOR_NUM_LITERAL, use_color);
            *offset += 1;
            format!("[{}]", ss)
        },
        0x1f => { 
            let v = cpu.mem[(cpu.pc + *offset) as usize];
            *offset += 1;
            maybe_colorize(format!("0x{:04x}", v), COLOR_NUM_LITERAL, use_color)
        },
        _ => {
            //format!("0x{:04x}", ((0x10000 + value - 0x21) & 0xffff) as i16)
            maybe_colorize(format!("{}", ((0x10000 + value - 0x21) & 0xffff) as i16),
                        COLOR_NUM_LITERAL, use_color)
        },
    }
}

fn registry_str(reg: usize) -> &'static str {
    match reg {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "X",
        4 => "Y",
        5 => "Z",
        6 => "I",
        7 => "J",
        _ => "?",
    }
}

pub fn disassemble_instruction(cpu: &DCPU, use_color: bool) -> (u16, String) {
    let mut offset = 1u16;
    let word = cpu.mem[cpu.pc as usize] as usize;
    let opcode = word & 0x1f;
    let id_b = (word >> 5) & 0x1f;
    let id_a = (word >> 10) & 0x3f;

    if opcode == 0 {
        let spec_opcode = (word >> 5) & 0x1f;
        let s_a = value_str(cpu, id_a, &mut offset, false, use_color);
        let ret = special_opcode_str(spec_opcode);
        match ret {
            Ok(s) => {
                let ss = maybe_colorize(s.to_string(), COLOR_INSTRUCTION, use_color);
                (offset, format!("{} {}", ss, s_a))
            },
            Err(_) => {
                let ss = maybe_colorize("DAT".to_string(), COLOR_INSTRUCTION, use_color);
                let vv = maybe_colorize(format!("0x{:04x}", word), COLOR_NUM_LITERAL, use_color);
                (offset, format!("{} {}", ss, vv))
            }
        }
    } else {
        let s_a = value_str(cpu, id_a, &mut offset, false, use_color);
        let s_b = value_str(cpu, id_b, &mut offset, true, use_color);
        let ret = opcode_str(opcode);
        match ret {
            Ok(s) => {
                let ss = maybe_colorize(s.to_string(), COLOR_INSTRUCTION, use_color);
                (offset, format!("{} {}, {}", ss, s_b, s_a))
            },
            Err(_) => {
                let ss = maybe_colorize("DAT".to_string(), COLOR_INSTRUCTION, use_color);
                let vv = maybe_colorize(format!("0x{:04x}", word), COLOR_NUM_LITERAL, use_color);
                (offset, format!("{} {}", ss, vv))
            }
        }
    }
}


