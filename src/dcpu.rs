#![allow(dead_code)]

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Result;

use instructions::*;

pub const MEMORY_SIZE: usize = 65536;

pub const REG_A: usize = 0;
pub const REG_B: usize = 1;
pub const REG_C: usize = 2;
pub const REG_X: usize = 3;
pub const REG_Y: usize = 4;
pub const REG_Z: usize = 5;
pub const REG_I: usize = 6;
pub const REG_J: usize = 7;

const SHOW_ROWS_RADIUS: usize = 1;

pub trait Hardware {
    fn info_hardware_id_upper(&self) -> u16;
    fn info_hardware_id_lower(&self) -> u16;
    fn info_manufacturer_id_upper(&self) -> u16;
    fn info_manufacturer_id_lower(&self) -> u16;
    fn info_version(&self) -> u16;
    fn process_interrupt(&mut self, cpu: &mut DCPU) -> ();

    fn get_data(&self, cpu: &DCPU) -> Vec<u8>;
}

// Some example Hardware implementations
// TODO: Possibly remove altogether from base repo
/*
pub struct HWMonitorLEM1802 {
    pub connected: bool,
    pub ram_location: u16,
}

impl Hardware for HWMonitorLEM1802 {
    fn info_hardware_id_upper(&self) -> u16 { 0x7349 }
    fn info_hardware_id_lower(&self) -> u16 { 0xf615 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x1c6c }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x8b36 }
    fn info_version(&self) -> u16 { 0x1802 }

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let a = cpu.reg[REG_A];
        let b = cpu.reg[REG_B];
        match a {
            0 => { /* MEM_MAP_SCREEN */
                if b > 0 {
                    self.ram_location = b;
                    self.connected = true;
                } else {
                    self.connected = false;
                }
            },
            _ => {}
        }
    }

    fn get_data(&self, _: &DCPU) -> Vec<u8> {
        Vec::new()
    }
}

const FLOPPY_MEMORY_SIZE: usize = 737280;
const FLOPPY_SECTOR_SIZE: usize = 512;

pub struct HWFloppyM35FDSector {
    pub mem: [u16; FLOPPY_SECTOR_SIZE],
}

pub struct HWFloppyM35FD {
    pub state: u16,
    pub error: u16,
    pub interrupt_message: u16,
    pub sectors: Vec<HWFloppyM35FDSector>,
}

const ERROR_NONE: u16       = 0x0000;
const ERROR_BUSY: u16       = 0x0001;
const ERROR_NO_MEDIA: u16   = 0x0002;
const ERROR_PROTECTED: u16  = 0x0003;
const ERROR_EJECT: u16      = 0x0004;
const ERROR_BAD_SECTOR: u16 = 0x0005;
const ERROR_BROKEN: u16     = 0xffff;

const STATE_NO_MEDIA: u16   = 0x0000;
const STATE_READY: u16      = 0x0001;
const STATE_READY_WP: u16   = 0x0002;
const STATE_BUSY: u16       = 0x0003;

impl HWFloppyM35FD {
    pub fn new() -> HWFloppyM35FD {
        HWFloppyM35FD {
            state: 0,
            error: 0,
            interrupt_message: 0,
            //mem: Box::new([0; FLOPPY_MEMORY_SIZE]),
            sectors: Vec::new(),
        }
    }
}

impl Hardware for HWFloppyM35FD {
    fn info_hardware_id_upper(&self) -> u16 { 0x4fd5 }
    fn info_hardware_id_lower(&self) -> u16 { 0x24c5 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x1eb3 }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x7e91 }
    fn info_version(&self) -> u16 { 0x000b }

    /*
    fn set_error(&mut self, cpu: &mut DCPU, err: u16) -> () {
        if err != self.error {
            println!("Trigger DCPU interrupt!");
        }
    }
    */

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let a = cpu.reg[REG_A];
        let (old_error, old_state) = (self.error, self.state);

        match a {
            0 => { /* Poll device */
                cpu.reg[REG_B] = self.state as u16;
                cpu.reg[REG_C] = self.error as u16;
                println!("Polling");
            },
            1 => { /* Set interrupt */
                let x = cpu.reg[REG_X] as usize;
                self.interrupt_message = x as u16;
            },
            2 => { /* Read sector */
                let x = cpu.reg[REG_X] as usize;
                let y = cpu.reg[REG_Y] as usize;

                match self.state {
                    STATE_NO_MEDIA => {
                        self.error = ERROR_NO_MEDIA;
                    },
                    STATE_READY | STATE_READY_WP => {
                        if x < 1440 {
                            // TODO: Synchronous reading, for now
                            match self.sectors.get(x) {
                                Some(s) => {
                                    for i in 0..512 {
                                        cpu.mem[(y + i) % MEMORY_SIZE] = s.mem[i];
                                    }
                                },
                                None => {
                                    for i in 0..512 {
                                        cpu.mem[(y + i) % MEMORY_SIZE] = 0;
                                    }
                                },
                            }
                            self.error = ERROR_NONE;
                        } else {
                            self.error = ERROR_BAD_SECTOR;
                        }

                    },
                    _ => {
                        self.error = ERROR_BROKEN;
                    }
                }
            },
            _ => {}
        }

        if self.state != old_state || self.error != old_error {
            if self.interrupt_message > 0 {
                println!("Trigger interrupt! {}", self.interrupt_message);
            }
        }
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
    pub devices: Vec<Box<Hardware>>,
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
            devices: Vec::new(),
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
                    self.sp = (((self.sp as usize) + 1) % MEMORY_SIZE) as u16;
                    self.mem[oldsp as usize]
                } else {
                    self.sp = (((self.sp as usize) + MEMORY_SIZE - 1) % MEMORY_SIZE) as u16;
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
                self.sp = (((self.sp as usize) + MEMORY_SIZE - 1) % MEMORY_SIZE) as u16;
                self.mem[self.sp as usize] = self.pc + 1;
                self.pc = self.get(id_a, true, true);
            },

            HWN => {
                self.cycle += 2;
                let n_devices = self.devices.len() as u16;
                self.set(id_a, n_devices);
            },
            HWQ => {
                self.cycle += 4;
                let device_id = self.get(id_a, true, true) as usize;
                //let n_devices = self.devices.len() as u16;
                //
                let (i1, i2, i3, i4, i5) = match self.devices.get(device_id) {
                    Some(d) => {
                        (d.info_hardware_id_lower(),
                         d.info_hardware_id_upper(),
                         d.info_version(),
                         d.info_manufacturer_id_lower(),
                         d.info_manufacturer_id_upper())
                    },
                    None => {
                        (0, 0, 0, 0, 0)
                    },
                };
                self.set(0, i1);
                self.set(1, i2);
                self.set(2, i3);
                self.set(3, i4);
                self.set(4, i5);
            },
            HWI => {
                self.cycle += 4;
                let device_id = self.get(id_a, true, true) as usize;
                // TODO: I have to remove the device and then add it back in
                //       I'm sure there is a better way to do this.
                //       Also, instead of this if statement, perhaps catch 
                //       the panic potentially returned by remove.
                if device_id < self.devices.len() {
                    let mut device = self.devices.remove(device_id);
                    device.process_interrupt(self);
                    self.devices.insert(device_id, device);
                }
            },
            /* Extensions */
            OUT => {
                // Temporary printing
                // OUT p  (prints memory address p as a null-terminated string)
                let a = self.get(id_a, true, true);
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

    pub fn load_from_assembly_file(&mut self, path: &Path) -> Result<()> {
        let mut file = try!(File::open(&path));
        let mut i = 0;
        let mut buffer: Vec<u8> = Vec::new();
        try!(file.read_to_end(&mut buffer));
        let mut j = 0;
        let mut sig = 0u16;
        for v in buffer {
            if j % 2 == 0 {
                sig = v as u16;
            } else {
                self.mem[i] = (sig << 8) + (v as u16);
                i += 1;
            }

            j += 1;
        }
        Ok(())
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

