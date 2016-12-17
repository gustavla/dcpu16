use dcpu::{self, DCPU, Device};
use std::any::Any;

// If the queue (buffer) fills up, it will start to drop old entries
const MAX_BUFFER: usize = 256;

fn with_shift(key: u16) -> u16 {
    match key {
        0x30 => 0x29,
        0x31 => 0x21,
        0x32 => 0x40,
        0x33 => 0x23,
        0x34 => 0x24,
        0x35 => 0x25,
        0x36 => 0x5e,
        0x37 => 0x26,
        0x38 => 0x2a,
        0x39 => 0x28,
        0x3b => 0x3a,
        0x2a => 0x3c,
        0x2e => 0x3e,
        0x2f => 0x3f,
        0x2d => 0x5f,
        0x3d => 0x2b,
        a @ 0x61 ... 0x7a => a - 32,
        a @ 0x5b ... 0x5d => a + 32,
        a => a,
    }
}

pub struct DeviceKeyboardGeneric {
    buffer: Vec<u16>,
    interrupt_message: Option<u16>,
    pressed: [bool; 0x92],
}

impl DeviceKeyboardGeneric {
    pub fn new() -> DeviceKeyboardGeneric {
        DeviceKeyboardGeneric {
            buffer: Vec::new(),
            interrupt_message: None,
            pressed: [false; 0x92],
        }
    }

    pub fn register_press(&mut self, cpu: &mut DCPU, key: u16) -> () {
        // Do not add shift/ctrl to queue
        if key != 0x90 && key != 0x91 {
            // Check if buffer has grown too big
            // If it is, we clear the whole buffer (let's pretend it fails)
            // This will likely happen when the user isn't even using the buffers, or if they turn
            // on queueing and let it go too long.
            if self.buffer.len() >= MAX_BUFFER {
                self.buffer.clear();
            }

            if self.pressed[0x90] {
                self.buffer.push(with_shift(key));
            } else {
                self.buffer.push(key);
            }
        }

        // Mark it as pressed
        if key < 0x92 {
            self.pressed[key as usize] = true;
        }

        // Trigger interrupt
        if let Some(m) = self.interrupt_message {
            cpu.interrupt(m);
        }
    }

    pub fn register_release(&mut self, cpu: &mut DCPU, key: u16) -> () {
        // Buffer is unaffected by this operation

        // Unmark it as pressed
        if key < 0x92 {
            self.pressed[key as usize] = false;
        }

        // Trigger interrupt
        if let Some(m) = self.interrupt_message {
            cpu.interrupt(m);
        }
    }
}

impl Device for DeviceKeyboardGeneric {
    fn info_hardware_id_upper(&self) -> u16 { 0x30cf }
    fn info_hardware_id_lower(&self) -> u16 { 0x7406 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x0 }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x0 }
    fn info_version(&self) -> u16 { 1 }

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let reg_a = cpu.reg[dcpu::REG_A];
        let reg_b = cpu.reg[dcpu::REG_B];
        match reg_a {
            0 => { // Clear keyboard buffer
                self.buffer.clear();
            },
            1 => {
                let v = self.buffer.pop().unwrap_or(0);
                cpu.reg[dcpu::REG_C] = v;
            },
            2 => {
                let key = cpu.reg[dcpu::REG_B];
                cpu.reg[dcpu::REG_C] = if key < 0x92 && self.pressed[key as usize] {
                    1
                } else {
                    0
                };
            },
            3 => {
                self.interrupt_message = if reg_b != 0 {
                    Some(reg_b)
                } else {
                    None
                };
            },
            _ => {}
        }
    }

    fn run(&mut self, _: &mut DCPU, _: usize) -> () {}

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}


