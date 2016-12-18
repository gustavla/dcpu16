use dcpu::{self, DCPU, Device};
use std::any::Any;

// Currently, the clock is based on DCPU-16 cycles, so it could be implemented with software on the
// DCPU-16 itself.

pub struct DeviceClockGeneric {
    //over_sixty_per_seconds: u16,
    cycles_between_ticks: Option<usize>,
    cycles_in_current_tick: usize,
    ticks: u16,
    interrupt_message: Option<u16>,
}

impl DeviceClockGeneric {
    pub fn new() -> DeviceClockGeneric {
        DeviceClockGeneric {
            cycles_between_ticks: None,
            cycles_in_current_tick: 0,
            ticks: 0,
            interrupt_message: None,
        }
    }
}

impl Device for DeviceClockGeneric {
    fn info_hardware_id_upper(&self) -> u16 { 0x12d0 }
    fn info_hardware_id_lower(&self) -> u16 { 0xb402 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x0 }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x0 }
    fn info_version(&self) -> u16 { 1 }

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let reg_a = cpu.reg[dcpu::REG_A];
        let reg_b = cpu.reg[dcpu::REG_B];
        match reg_a {
            0 => { // Set clock tick
                if reg_b == 0 {
                    self.cycles_between_ticks = None;
                } else {
                    self.cycles_between_ticks = Some(((dcpu::CYCLE_HZ as f64) / (60.0 / (reg_b as f64))) as usize);
                }
                self.cycles_in_current_tick = 0;
                self.ticks = 0;
            },
            1 => { // Query number of ticks
                cpu.reg[dcpu::REG_C] = self.ticks;
            },
            2 => { // Set interrupt
                self.interrupt_message = if reg_b != 0 {
                    Some(reg_b)
                } else {
                    None
                };
            },
            _ => {}
        }
    }

    fn run(&mut self, cpu: &mut DCPU, cycles: usize) -> () {
        match self.cycles_between_ticks {
            Some(n_cycles) => {
                self.cycles_in_current_tick += cycles;
                while self.cycles_in_current_tick >= n_cycles {
                    self.cycles_in_current_tick -= n_cycles;
                    self.ticks += 1;
                    if let Some(m) = self.interrupt_message {
                        cpu.interrupt(m);
                    }
                }
            },
            None => {
                // Clock is turned off
            },
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
