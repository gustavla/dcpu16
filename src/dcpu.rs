#![allow(dead_code)]

use instructions::*;

pub const MEMORY_SIZE: usize = 65536;

const SHOW_ROWS_RADIUS: usize = 1;

/*
trait Hardware {
    fn info_hardware_id_upper() -> usize;
    fn info_hardware_id_lower() -> usize;
    fn info_manufacturer_id_upper() -> usize;
    fn info_manufacturer_id_lower() -> usize;
    fn info_version() -> usize;
    fn process_interrupt(cpu: &mut DCPU) -> ();
}

pub struct HWMonitorLEM1802 {
    connected: bool,
    ram_location: u16,
}

impl Hardware for HWMonitorLEM1802 {
    fn info_hardware_id_upper() -> usize { 0x7349 }
    fn info_hardware_id_lower() -> usize { 0xf615 }
    fn info_manufacturer_id_upper() -> usize { 0x1c6c }
    fn info_manufacturer_id_lower() -> usize { 0x8b36 }
    fn info_version() -> usize { 0x1802 }

    fn process_interrupt(cpu: &mut DCPU) -> () {

    }
}
*/

pub struct DCPU {
    pub terminate: bool,
    pub reg: [u16; 8],
    pub mem: [u16; MEMORY_SIZE],
    pub pc: u16,
    sp: u16,
    ex: u16,
    ia: u16,
    skip_next: bool,
    cycle: usize,
    //devices: Vec<Hardware + 'static>,
}

impl DCPU {
    pub fn new() -> DCPU {
        DCPU {
            terminate: false,
            reg: [0; 8],
            mem: [0; MEMORY_SIZE],
            pc: 0,
            sp: 0,
            ex: 0,
            ia: 0,
            skip_next: false,
            cycle: 0,
            //devices: vec!(),
        }
    }

    /*
    fn reset(&mut self) {
        self.terminate = false;
        for i in 0..65536 {
            self.mem[i] = 0;
        }
        for i in 0..8 {
            self.reg[i] = 0;
        }
        self.pc = 0;
        self.sp = 0;
        self.ex = 0;
        self.ia = 0;
        self.cycle = 0;
        self.skip_next = false;
    }
    */

    fn pcplus(&mut self, movepc: bool) -> u16 {
        let oldpc = self.pc;
        if movepc {
            if self.pc == 0xffff {
                self.pc = 0;
            } else {
                self.pc += 1;
            }
        }
        oldpc
    }

    fn set(&mut self, identifier: usize, value: u16) {
        match identifier {
            0x00 ... 0x07 => { self.reg[identifier] = value; },
            0x08 ... 0x0f => {
                self.cycle += 1;
                let pos = self.reg[(identifier - 0x08) as usize];
                self.mem[pos as usize] = value;
            },
            0x10 ... 0x17 => {
                self.cycle += 1;
                let pos = self.reg[(identifier - 0x10) as usize];
                let offset = self.mem[self.pcplus(true) as usize];
                self.mem[(pos + offset) as usize] = value;
            },
            0x18 => {
                self.sp -= 1;
                self.mem[self.sp as usize] = value;
            },
            0x19 => {
                self.mem[self.sp as usize] = value;
            },
            0x1a => {
                self.cycle += 1;
                let pos = self.sp + self.mem[self.pcplus(true) as usize];
                self.mem[pos as usize] = value;
            },
            0x1b => { self.sp = value; },
            0x1c => { self.pc = value; },
            0x1d => { self.ex = value; },
            0x1e => {
                self.cycle += 1;
                let pos = self.mem[self.pcplus(true) as usize];
                self.mem[pos as usize] = value;
            }
            // Instructions 0x1f - 0x3f are not possible (silently ignore)
            _ => {},
        }
    }

    fn get(&mut self, identifier: usize, is_a: bool, movepc: bool) -> u16 {
        match identifier {
            0x00 ... 0x07  => { self.reg[identifier as usize] },
            0x08 ... 0x0f => {
                let pos = self.reg[(identifier - 0x08) as usize];
                self.mem[pos as usize]
            },
            0x10 ... 0x17 => {
                self.cycle += 1;
                let pos = self.reg[(identifier - 0x10) as usize];
                let offset = self.mem[self.pcplus(movepc) as usize];
                self.mem[(pos + offset) as usize]
            },
            0x18 => {
                if is_a {
                    let oldsp = self.sp;
                    self.sp += 1;
                    self.mem[oldsp as usize]
                } else {
                    self.sp -= 1;
                    self.mem[self.sp as usize]
                }
            },
            0x19 => { self.mem[self.sp as usize] },
            0x1a => {
                self.cycle += 1;
                let pos = self.sp + self.mem[self.pcplus(movepc) as usize];
                self.mem[pos as usize]
            },
            0x1b => { self.sp },
            0x1c => { self.pc },
            0x1d => { self.ex },
            0x1e => {
                self.cycle += 1;
                let pos = self.mem[self.pcplus(movepc) as usize];
                self.mem[pos as usize]
            },
            0x1f => {
                self.cycle += 1;
                self.mem[self.pcplus(movepc) as usize]
            },
            _ => {
                if is_a && identifier >= 0x20 && identifier <= 0x3f {
                    ((0x10000 + identifier - 0x21) & 0xffff) as u16
                } else {
                    0
                }
            },
        }
    }

    fn get_signed(&mut self, identifier: usize, is_a: bool, movepc: bool) -> i16 {
        let v = self.get(identifier, is_a, movepc);
        v as i16
    }

    fn takes_next(&self, id: usize) -> bool {
        id >= 0x10 && id <= 0x17 || id == 0x1a || id == 0x1e || id == 0x1f
    }

    fn process_conditional(&mut self, truth: bool) {
        if !truth {
            self.skip_next = true;
            self.cycle += 1;
        }
    }

    pub fn tick(&mut self) {
        let word = self.mem[self.pcplus(true) as usize] as usize;
        let opcode = word & 0x1f;
        let id_b = (word >> 5) & 0x1f;
        let id_a = (word >> 10) & 0x3f;

        if self.skip_next {
            if self.takes_next(id_a) {
                self.pcplus(true);
            }

            if opcode != 0 && self.takes_next(id_b) {
                self.pcplus(true);
            }

            self.skip_next = opcode >= 0x10 && opcode <= 0x17;
            if self.skip_next {
                self.cycle += 1;
            }
        } else {
            match opcode {
                0 => {
                    let spec_opcode = (word >> 5) & 0x1f;
                    self.process_special_opcode(spec_opcode, id_a);
                },
                SET => {
                    self.cycle += 1;
                    let v = self.get(id_a, true, true);
                    self.set(id_b, v);
                },
                ADD => {
                    self.cycle += 2;
                    let v = (self.get(id_a, true, true) as i32) +
                            (self.get(id_b, false, false) as i32);
                    if v >= 0xffff {
                        self.ex = 1;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16); // & might not be needed
                },
                SUB => {
                    self.cycle += 2;
                    let a = self.get(id_a, true, true);
                    let b = self.get(id_b, false, false);
                    if a > b {
                        self.ex = 0xffff;
                    } else {
                        self.ex = 0;
                    }
                    let v = b - a;
                    self.set(id_b, v);
                },
                MUL => {
                    self.cycle += 2;
                    let v = (self.get(id_a, true, true) as i32) *
                            (self.get(id_b, false, false) as i32);
                    if v > 0xffff || v < 0 {
                        self.ex = ((v >> 16) & 0xffff) as u16;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16);
                },
                MLI => {
                    self.cycle += 2;
                    let v = (self.get_signed(id_a, true, true) as i32) *
                            (self.get_signed(id_b, false, false) as i32);
                    if v > 0xffff || v < 0 {
                        self.ex = ((v >> 16) & 0xffff) as u16;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16);
                },
                DIV => {
                    self.cycle += 3;
                    let a = self.get(id_a, true, true);
                    let b = self.get(id_b, false, false);
                    let v = if a == 0 {
                        self.ex = 0;
                        0u16
                    } else {
                        self.ex = ((((b as i32) << 16) / (a as i32)) & 0xffff) as u16;
                        b / a
                    };
                    self.set(id_b, v);
                },
                DVI => {
                    self.cycle += 3;
                    let a = self.get_signed(id_a, true, true);
                    let b = self.get_signed(id_b, false, false);
                    let v = if a == 0 {
                        self.ex = 0;
                        0i16
                    } else {
                        self.ex = ((((b as i32) << 16) / (a as i32)) & 0xffff) as u16;
                        b / a
                    };
                    self.set(id_b, v as u16);
                },
                MOD => {
                    self.cycle += 3;
                    let a = self.get(id_a, true, true);
                    let b = self.get(id_b, false, false);
                    let v = if a != 0 {
                        b % a
                    } else {
                        0
                    };
                    self.set(id_b, v);
                },
                MDI => {
                    self.cycle += 3;
                    let a = self.get_signed(id_a, true, true);
                    let b = self.get_signed(id_b, false, false);
                    let v = if a != 0 {
                        b % a
                    } else {
                        0
                    };
                    self.set(id_b, v as u16);
                },
                AND => {
                    self.cycle += 1;
                    let v = self.get(id_a, true, true) & self.get(id_b, false, false);
                    self.set(id_b, v);
                },
                BOR => {
                    self.cycle += 1;
                    let v = self.get(id_a, true, true) | self.get(id_b, false, false);
                    self.set(id_b, v);
                },
                XOR => {
                    self.cycle += 1;
                    let v = self.get(id_a, true, true) ^ self.get(id_b, false, false);
                    self.set(id_b, v);
                },
                SHR => {
                    self.cycle += 1;
                    let a = self.get(id_a, true, true);
                    let b = self.get(id_b, false, false);
                    let v = b >> a;
                    self.ex = (((b as u32) << 16)>>a) as u16;
                    self.set(id_b, v);
                },
                ASR => {
                    self.cycle += 1;
                    let a = self.get(id_a, true, true);
                    let b = self.get_signed(id_b, false, false);
                    let v = (b >> a) as u16;
                    self.ex = (((b as u32) << 16)>> (a as u32)) as u16;
                    self.set(id_b, v);
                },
                SHL => {
                    self.cycle += 1;
                    let a = self.get(id_a, true, true);
                    let b = self.get(id_b, false, false);
                    self.ex = (((b as u32) << (a as u32)) >> 16) as u16;
                    let v = b << a;
                    self.set(id_b, v);
                },
                IFB => {
                    self.cycle += 2;
                    let truth = (self.get(id_b, false, true) & self.get(id_a, true, true)) != 0;
                    self.process_conditional(truth);
                },
                IFC => {
                    self.cycle += 2;
                    let truth = (self.get(id_b, false, true) & self.get(id_a, true, true)) == 0;
                    self.process_conditional(truth);
                },
                IFE => {
                    self.cycle += 2;
                    let truth = self.get(id_b, false, true) == self.get(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFN => {
                    self.cycle += 2;
                    let truth = self.get(id_b, false, true) != self.get(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFG => {
                    self.cycle += 2;
                    let truth = self.get(id_b, false, true) > self.get(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFA => {
                    self.cycle += 2;
                    let truth = self.get_signed(id_b, false, true) > self.get_signed(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFL => {
                    self.cycle += 2;
                    let truth = self.get(id_b, false, true) < self.get(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFU => {
                    self.cycle += 2;
                    let truth = self.get_signed(id_b, false, true) < self.get_signed(id_a, true, true);
                    self.process_conditional(truth);
                },
                /* TODO: ADX, SBX, STI, STD */
                ADX => {

                },
                SBX => {

                },
                STI => {

                },
                STD => {

                },
                _ => {},
            }
        }
        if self.skip_next {
            self.tick();
        }
    }

    fn process_special_opcode(&mut self, spec_opcode: usize, id_a: usize) {
        match spec_opcode {
            0 => {
                // Terminate if a 0x00 is processed as an instruction
                self.terminate = true;
            },
            JSR => {
                self.cycle += 3;
                self.sp -= 1;
                self.mem[self.sp as usize] = self.pc + 1;
                self.pc = self.get(id_a, true, true);
            },
            OUT => {
                // Temporary printing
                // OUT p  (prints memory address p as a null-terminated string)
                let a = self.get(id_a, true, true);
                // TODO: The following needs parens for some reason
                for i in 0..MEMORY_SIZE {
                    let c = self.mem[((a + i as u16) as u16) as usize];
                    if c == 0 {
                        break;
                    } else {
                        print!("{}", ((c & 0xff) as u8) as char);
                    }
                }
            }
            _ => {},
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("PC {:04x} SP {:04x} IA {:04x} EX {:04x}\n\
                  A  {:04x} B  {:04x} C  {:04x}\n\
                  X  {:04x} Y  {:04x} Z  {:04x}\n\
                  I  {:04x} J  {:04x}                cycles: {:6}", self.pc,
                 self.sp, self.ia, self.ex, self.reg[0], self.reg[1], self.reg[2], self.reg[3],
                 self.reg[4], self.reg[5], self.reg[6], self.reg[7], self.cycle
                );

        // Determine context window
        let rows = MEMORY_SIZE / 8;
        let window = 2 * SHOW_ROWS_RADIUS + 1;
        let cur_row = (self.pc / 8) as usize;
        let (from, to) = if cur_row < SHOW_ROWS_RADIUS {
            (0, window)
        } else if cur_row >= rows - SHOW_ROWS_RADIUS {
            (rows - window, rows)
        } else {
            (cur_row - SHOW_ROWS_RADIUS, cur_row + SHOW_ROWS_RADIUS + 1)
        };

        for i in from..to {
            let p = i * 8;
            print!("{:04x}: ", p);
            for j in 0..8usize {
                if self.pc as usize == p + j {
                    print!("\x1b[32m{:04x}\x1b[0m ", self.mem[(p + j) as usize]);
                } else {
                    print!("{:04x} ", self.mem[(p + j) as usize]);
                }
            }
            println!("");
        }
    }
}

