#![allow(dead_code)]

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use instructions::*;

// Note: this can't be changed willy-nilly, since the PC is naturally wrapped around, so it will
// not wrap around correctly if this is changed.
pub const MEMORY_SIZE: usize = 0x10000;

pub const REG_A: usize = 0;
pub const REG_B: usize = 1;
pub const REG_C: usize = 2;
pub const REG_X: usize = 3;
pub const REG_Y: usize = 4;
pub const REG_Z: usize = 5;
pub const REG_I: usize = 6;
pub const REG_J: usize = 7;

const SHOW_ROWS_RADIUS: usize = 1;

pub trait Device {
    fn info_hardware_id_upper(&self) -> u16;
    fn info_hardware_id_lower(&self) -> u16;
    fn info_manufacturer_id_upper(&self) -> u16;
    fn info_manufacturer_id_lower(&self) -> u16;
    fn info_version(&self) -> u16;
    fn process_interrupt(&mut self, cpu: &mut DCPU) -> ();
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

pub struct DCPU {
    pub terminate: bool,
    pub reg: [u16; 8],
    pub mem: [u16; MEMORY_SIZE],
    pub pc: u16,
    pub sp: u16,
    pub ex: u16,
    pub ia: u16,
    interrupt_queueing: bool,
    interrupt_queue: Vec<u16>,
    skip_next: bool,
    cycle: usize,
    overshot_cycles: isize,
    inside_run: bool,
    pub devices: Rc<Vec<RefCell<Box<Device>>>>,
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
            interrupt_queueing: false,
            interrupt_queue: Vec::new(),
            skip_next: false,
            cycle: 0,
            overshot_cycles: 0,
            inside_run: false,
            devices: Rc::new(Vec::new()),
        }
    }

    // Run multiple ticks until cycles have been met
    // Resets cycle count, so that it won't overflow
    pub fn run(&mut self, cycles: usize) {
        self.inside_run = true;
        if self.overshot_cycles > cycles as isize {
            self.overshot_cycles -= cycles as isize;
            return;
        }

        let end_cycle = ((self.cycle + cycles) as isize - self.overshot_cycles) as usize;

        while self.cycle < end_cycle {
            self.tick();
        }
        self.overshot_cycles = self.cycle as isize - end_cycle as isize;

        // Pretend cycle is u16 and let it overflow safely
        while self.cycle > 0xffff {
            self.cycle -= 0xffff;
        }
        self.inside_run = false;
    }

    /// Get cycle count
    pub fn cycle(&self) -> usize {
        self.cycle
    }

    /// Halts the DCPU for a specified number of cycles.
    pub fn halt(&mut self, cycles: usize) -> () {
        if self.inside_run {
            self.cycle += cycles;
        } else {
            self.overshot_cycles += cycles as isize;
        }
    }

    fn reset(&mut self) {
        self.terminate = false;
        for i in 0..MEMORY_SIZE {
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
        self.interrupt_queue = Vec::new();
        self.interrupt_queueing = false;
        self.overshot_cycles = 0;
        self.skip_next = false;
        self.devices = Rc::new(Vec::new());
    }

    fn pcplus(&mut self, movepc: bool) -> u16 {
        let oldpc = self.pc;
        if movepc {
            self.pc = self.pc.wrapping_add(1);
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
                self.mem[pos.wrapping_add(offset) as usize] = value;
            },
            0x18 => {
                self.sp = self.sp.wrapping_sub(1);
                self.mem[self.sp as usize] = value;
            },
            0x19 => {
                self.mem[self.sp as usize] = value;
            },
            0x1a => {
                self.cycle += 1;
                let pos = self.sp.wrapping_add(self.mem[self.pcplus(true) as usize]);
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

    fn value(&mut self, identifier: usize, is_a: bool, movepc: bool) -> u16 {
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
                self.mem[pos.wrapping_add(offset) as usize]
            },
            0x18 => {
                if is_a {
                    let oldsp = self.sp;
                    self.sp = self.sp.wrapping_add(1);
                    self.mem[oldsp as usize]
                } else {
                    self.sp = self.sp.wrapping_sub(1);
                    self.mem[self.sp as usize]
                }
            },
            0x19 => { self.mem[self.sp as usize] },
            0x1a => {
                self.cycle += 1;
                let pos = self.sp.wrapping_add(self.mem[self.pcplus(movepc) as usize]);
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

    fn value_signed(&mut self, identifier: usize, is_a: bool, movepc: bool) -> i16 {
        let v = self.value(identifier, is_a, movepc);
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

    /// Connect device
    pub fn add_device(&mut self, device: Box<Device>) -> () {
        if let Some(devices) = Rc::get_mut(&mut self.devices) {
            devices.push(RefCell::new(device));
        }
    }

    /// Queues up interrupt. Can be used from hardware.
    pub fn interrupt(&mut self, message: u16) -> () {
        self.interrupt_queue.push(message);
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
                    let v = self.value(id_a, true, true);
                    self.set(id_b, v);
                },
                ADD => {
                    self.cycle += 2;
                    // TODO: Use overflow_add
                    let v = (self.value(id_a, true, true) as i32) +
                            (self.value(id_b, false, false) as i32);
                    if v > 0xffff {
                        self.ex = 1;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16); // & might not be needed
                },
                SUB => {
                    self.cycle += 2;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    if a > b {
                        self.ex = 0xffff;
                    } else {
                        self.ex = 0;
                    }
                    let v = b.wrapping_sub(a);
                    self.set(id_b, v);
                },
                MUL => {
                    self.cycle += 2;
                    let v = (self.value(id_a, true, true) as i32) *
                            (self.value(id_b, false, false) as i32);
                    if v > 0xffff || v < 0 {
                        self.ex = ((v >> 16) & 0xffff) as u16;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16);
                },
                MLI => {
                    self.cycle += 2;
                    let v = (self.value_signed(id_a, true, true) as i32) *
                            (self.value_signed(id_b, false, false) as i32);
                    if v > 0xffff || v < 0 {
                        self.ex = ((v >> 16) & 0xffff) as u16;
                    } else {
                        self.ex = 0;
                    }
                    self.set(id_b, (v & 0xffff) as u16);
                },
                DIV => {
                    self.cycle += 3;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
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
                    let a = self.value_signed(id_a, true, true);
                    let b = self.value_signed(id_b, false, false);
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
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    let v = if a != 0 {
                        b % a
                    } else {
                        0
                    };
                    self.set(id_b, v);
                },
                MDI => {
                    self.cycle += 3;
                    let a = self.value_signed(id_a, true, true);
                    let b = self.value_signed(id_b, false, false);
                    let v = if a != 0 {
                        b % a
                    } else {
                        0
                    };
                    self.set(id_b, v as u16);
                },
                AND => {
                    self.cycle += 1;
                    let v = self.value(id_a, true, true) & self.value(id_b, false, false);
                    self.set(id_b, v);
                },
                BOR => {
                    self.cycle += 1;
                    let v = self.value(id_a, true, true) | self.value(id_b, false, false);
                    self.set(id_b, v);
                },
                XOR => {
                    self.cycle += 1;
                    let v = self.value(id_a, true, true) ^ self.value(id_b, false, false);
                    self.set(id_b, v);
                },
                // TODO: These can panic (in debug mode) if shifting too much (>=16)
                SHR => {
                    self.cycle += 1;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    let v = b >> a;
                    self.ex = (((b as u32) << 16)>>a) as u16;
                    self.set(id_b, v);
                },
                ASR => {
                    self.cycle += 1;
                    let a = self.value(id_a, true, true);
                    let b = self.value_signed(id_b, false, false);
                    let v = (b >> a) as u16;
                    self.ex = (((b as u32) << 16)>> (a as u32)) as u16;
                    self.set(id_b, v);
                },
                SHL => {
                    self.cycle += 1;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    self.ex = (((b as u32) << (a as u32)) >> 16) as u16;
                    let v = b << a;
                    self.set(id_b, v);
                },
                IFB => {
                    self.cycle += 2;
                    let truth = (self.value(id_b, false, true) & self.value(id_a, true, true)) != 0;
                    self.process_conditional(truth);
                },
                IFC => {
                    self.cycle += 2;
                    let truth = (self.value(id_b, false, true) & self.value(id_a, true, true)) == 0;
                    self.process_conditional(truth);
                },
                IFE => {
                    self.cycle += 2;
                    let truth = self.value(id_b, false, true) == self.value(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFN => {
                    self.cycle += 2;
                    let truth = self.value(id_b, false, true) != self.value(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFG => {
                    self.cycle += 2;
                    let truth = self.value(id_b, false, true) > self.value(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFA => {
                    self.cycle += 2;
                    let truth = self.value_signed(id_b, false, true) > self.value_signed(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFL => {
                    self.cycle += 2;
                    let truth = self.value(id_b, false, true) < self.value(id_a, true, true);
                    self.process_conditional(truth);
                },
                IFU => {
                    self.cycle += 2;
                    let truth = self.value_signed(id_b, false, true) < self.value_signed(id_a, true, true);
                    self.process_conditional(truth);
                },
                ADX => {
                    self.cycle += 3;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    if (a as usize) + (b as usize) + (self.ex as usize) > 0xffff {
                        self.ex = 1;
                    } else {
                        self.ex = 0;
                    }
                    let v = a.wrapping_add(b).wrapping_add(self.ex);
                    self.set(id_b, v);
                },
                SBX => {
                    self.cycle += 3;
                    let a = self.value(id_a, true, true);
                    let b = self.value(id_b, false, false);
                    if (a as usize) + (self.ex as usize) < (b as usize) {
                        self.ex = 0xffff;
                    } else {
                        self.ex = 0;
                    }
                    let v = a.wrapping_sub(b).wrapping_add(self.ex);
                    self.set(id_b, v);
                },
                STI => {
                    self.cycle += 2;
                    let v = self.value(id_a, true, true);
                    self.set(id_b, v);
                    // Increment I and J
                    let v_i = self.reg[REG_I];
                    let v_j = self.reg[REG_J];
                    self.reg[REG_I] = v_i.wrapping_add(1);
                    self.reg[REG_J] = v_j.wrapping_add(1);
                },
                STD => {
                    self.cycle += 2;
                    let v = self.value(id_a, true, true);
                    self.set(id_b, v);
                    // Decrement I and J
                    let v_i = self.reg[REG_I];
                    let v_j = self.reg[REG_J];
                    self.reg[REG_I] = v_i.wrapping_sub(1);
                    self.reg[REG_J] = v_j.wrapping_sub(1);
                },
                _ => {},
            }
        }
        if self.skip_next {
            self.tick();
        } else if !self.interrupt_queue.is_empty() && !self.interrupt_queueing {
            let message = self.interrupt_queue.remove(0);
            self.cycle += 4;

            if self.ia != 0 {
                self.interrupt_queueing = true;
                self.sp = self.sp.wrapping_sub(1);
                self.mem[self.sp as usize] = self.pc;
                self.sp = self.sp.wrapping_sub(1);
                self.mem[self.sp as usize] = self.reg[REG_A];

                self.pc = self.ia;
                self.reg[REG_A] = message;
            }
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
                self.sp = self.sp.wrapping_sub(1);
                let new_pc = self.value(id_a, true, true);
                self.mem[self.sp as usize] = self.pc;
                self.pc = new_pc;
            },
            INT => {
                let message = self.value(id_a, true, true);
                self.interrupt(message);
            },
            IAG => {
                self.cycle += 1;
                let ia = self.ia;
                self.set(id_a, ia);
            },
            IAS => {
                self.cycle += 1;
                self.ia = self.value(id_a, true, true);
            },
            RFI => {
                self.cycle += 3;
                self.interrupt_queueing = false;
                let stack_a = self.mem[self.sp as usize];
                self.reg[REG_A] = stack_a;
                self.sp = self.sp.wrapping_add(1);
                self.pc = self.mem[self.sp as usize];
                self.sp = self.sp.wrapping_add(1);
            },
            IAQ => {
                self.cycle += 2;
                let a = self.value(id_a, true, true);
                self.interrupt_queueing = a > 0;
            },
            HWN => {
                self.cycle += 2;
                let n_devices = self.devices.len() as u16;
                self.set(id_a, n_devices);
            },
            HWQ => {
                self.cycle += 4;
                let device_id = self.value(id_a, true, true) as usize;
                //let n_devices = self.devices.len() as u16;
                //
                let (i1, i2, i3, i4, i5) = match self.devices.get(device_id) {
                    Some(dref) => {
                        let d = dref.borrow();
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
                self.reg[REG_A] = i1;
                self.reg[REG_B] = i2;
                self.reg[REG_C] = i3;
                self.reg[REG_X] = i4;
                self.reg[REG_Y] = i5;
            },
            HWI => {
                self.cycle += 4;
                let device_id = self.value(id_a, true, true) as usize;
                if device_id < self.devices.len() {
                    let devices = self.devices.clone();
                    let mut device = devices.get(device_id).unwrap().borrow_mut();
                    device.process_interrupt(self);
                }
            },
            // Extensions
            OUT => {
                // Temporary printing
                // OUT p  (prints memory address p as a null-terminated string)
                let a = self.value(id_a, true, true);
                for i in 0..MEMORY_SIZE {
                    let c = self.mem[a.wrapping_add(i as u16) as usize];
                    if c == 0 {
                        break;
                    } else {
                        print!("{}", ((c & 0xff) as u8) as char);
                    }
                }
            }
            OUV => {
                let a = self.value(id_a, true, true);
                println!("{}", a);
            },
            _ => {},
        }
    }

    /// Loads a binary file into the memory. The file needs to be assembled separately.
    pub fn load_from_binary_file(&mut self, path: &Path) -> Result<()> {
        //let mut file = try!(File::open(&path));
        let mut file = File::open(&path)?;
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

    // This function will assemble the binary for you, so the input file should be a .dasm16 file.
    //pub fn load_from_assembly_file(&mut self, path: &Path) -> Result<()> {
        // TODO
    //}

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

        /*
        // Use this code to print the bottom (the stack)
        for i in (0xffff-8)/8..(0x10000/8) {
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
        */
    }
}
