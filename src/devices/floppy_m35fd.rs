use dcpu::{self, DCPU, Device};
use std::any::Any;
//use std::cmp;

const FLOPPY_SECTOR_SIZE: usize = 512;
const FLOPPY_NUM_SECTORS: usize = 1440;

// TODO: Calculate these from dcpu::CYCLE_HZ

// Read/write speed is 30700 words/second
// The DCPU-16 runs at 100 kHz, so a sector of size 512 words will take
// 512 / 30700 * 100000 = 1667 cycles to complete
const READ_WRITE_WAIT_CYCLES: usize = 1667;

// Cycles per track change (2.4 ms)
const READ_WRITE_TRACK_SEEK_CYCLES: usize = 240;

pub struct FloppyDisk {
    pub sectors: Vec<[u16; FLOPPY_SECTOR_SIZE]>,
    pub write_protected: bool,
}

impl FloppyDisk {
    pub fn new() -> FloppyDisk {
        FloppyDisk {
            sectors: Vec::new(),
            write_protected: false,
        }
    }
}

// This state is an implementation detail not related to the emulation
#[derive(Copy, Clone, Debug)]
enum FloppyInternalState {
    Idle,
    WaitToRead,
    WaitToWrite,
}

pub struct DeviceFloppyM35FD {
    state: u16,
    error: u16,
    pub interrupt_message: u16,
    pub disk: Option<FloppyDisk>,

    internal_state: FloppyInternalState,
    rw_sector: u16, // Also doubles as "last sector" before read/write
    rw_dcpu_address: u16,
    rw_wait_cycles: usize,

    interrupt_queued: bool,
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

impl DeviceFloppyM35FD {
    pub fn new() -> DeviceFloppyM35FD {
        DeviceFloppyM35FD {
            state: 0,
            error: 0,
            interrupt_message: 0,
            disk: None,

            internal_state: FloppyInternalState::Idle,
            rw_sector: 0, // Floppies generally start a cylinder 0
            rw_dcpu_address: 0x0,
            rw_wait_cycles: 0,

            interrupt_queued: false,
        }
    }

    pub fn state(&self) -> u16 {
        self.state
    }

    pub fn set_state(&mut self, state: u16) -> () {
        if self.state != state {
            self.interrupt_queued = true;
        }
        self.state = state;
    }

    pub fn error(&self) -> u16 {
        self.error
    }

    pub fn set_error(&mut self, error: u16) -> () {
        if self.error != error {
            self.interrupt_queued = true;
        }
        self.error = error;
    }

    /*
    pub fn has_disk(&self) -> bool {
        self.disk.is_some()
    }
    */

    pub fn insert(&mut self, disk: FloppyDisk) {
        // TODO: Check to see if floppy is already inserted?
        self.disk = Some(disk);
        self.set_state(STATE_READY);
    }

    pub fn eject(&mut self) -> Option<FloppyDisk> {
        self.set_state(STATE_NO_MEDIA);
        self.disk.take()
    }
}

impl Device for DeviceFloppyM35FD {
    fn info_hardware_id_upper(&self) -> u16 { 0x4fd5 }
    fn info_hardware_id_lower(&self) -> u16 { 0x24c5 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x1eb3 }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x7e91 }
    fn info_version(&self) -> u16 { 0x000b }

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let a = cpu.reg[dcpu::REG_A];
        match a {
            0 => { // Poll device
                cpu.reg[dcpu::REG_B] = self.state() as u16;
                cpu.reg[dcpu::REG_C] = self.error() as u16;
                self.error = ERROR_NONE;
            },
            1 => { // Set interrupt
                let x = cpu.reg[dcpu::REG_X];
                self.interrupt_message = x;
            },
            2 => { // Read sector
                let x = cpu.reg[dcpu::REG_X];
                let y = cpu.reg[dcpu::REG_Y];

                let error = match self.state() {
                    STATE_NO_MEDIA => ERROR_NO_MEDIA,
                    STATE_BUSY => ERROR_BUSY,
                    STATE_READY | STATE_READY_WP => {
                        if (x as usize) < FLOPPY_NUM_SECTORS {
                            // Issue sector read, will be performed after delay in run()
                            self.internal_state = FloppyInternalState::WaitToRead;
                            let track1 = (self.rw_sector / 18) as isize;
                            self.rw_sector = x;
                            let track2 = (self.rw_sector / 18) as isize;
                            self.rw_dcpu_address = y;
                            self.rw_wait_cycles = READ_WRITE_WAIT_CYCLES;
                            let track_diff = (track1 - track2).abs() as usize;
                            // Code if we want to switch to two-sided floppy with 160 tracks
                            // We assume a two-sided floppy (without independent seeks), so seek
                            // distance is zero between tracks 0 and 79, and similarly 5 and 75.
                            //let track_diff = cmp::min((track1 - track2).abs(),
                            //                          (159 - track1 - track2).abs()) as usize;
                            self.rw_wait_cycles += track_diff * READ_WRITE_TRACK_SEEK_CYCLES;
                            self.set_state(STATE_BUSY);
                            ERROR_NONE
                        } else {
                            ERROR_BAD_SECTOR
                        }
                    },
                    _ => ERROR_BROKEN,
                };
                self.set_error(error);
            },
            3 => { // Write sector
                let x = cpu.reg[dcpu::REG_X];
                let y = cpu.reg[dcpu::REG_Y];

                let error = match self.state() {
                    STATE_NO_MEDIA => ERROR_NO_MEDIA,
                    STATE_READY_WP => ERROR_PROTECTED,
                    STATE_BUSY => ERROR_BUSY,
                    STATE_READY => {
                        if (x as usize) < FLOPPY_NUM_SECTORS {
                            // Issue sector write, will be performed after delay in run()
                            self.internal_state = FloppyInternalState::WaitToWrite;
                            self.rw_sector = x;
                            self.rw_dcpu_address = y;
                            self.rw_wait_cycles = READ_WRITE_WAIT_CYCLES;
                            self.set_state(STATE_BUSY);
                            ERROR_NONE
                        } else {
                            ERROR_BAD_SECTOR
                        }
                    },
                    _ => ERROR_BROKEN,
                };
                self.set_error(error);
            }
            _ => {
                // Do nothing
            },
        }
    }

    fn run(&mut self, cpu: &mut DCPU, cycles: usize) -> () {
        match self.internal_state {
            FloppyInternalState::WaitToRead => {
                if self.rw_wait_cycles > cycles {
                    self.rw_wait_cycles -= cycles;
                } else {
                    self.internal_state = FloppyInternalState::Idle;
                    self.rw_wait_cycles = 0;
                    let mut state = self.state();
                    let error;
                    match self.disk {
                        Some(ref floppy_disk) => {
                            match floppy_disk.sectors.get(self.rw_sector as usize) {
                                Some(s) => {
                                    for i in 0..FLOPPY_SECTOR_SIZE {
                                        cpu.mem[self.rw_dcpu_address.wrapping_add(i as u16) as usize] = s[i];
                                    }
                                },
                                None => {
                                    for i in 0..FLOPPY_SECTOR_SIZE {
                                        cpu.mem[self.rw_dcpu_address.wrapping_add(i as u16) as usize] = 0;
                                    }
                                },
                            }
                            state = match floppy_disk.write_protected {
                                true => STATE_READY_WP,
                                false => STATE_READY,
                            };
                            error = ERROR_NONE;
                        },
                        None => {
                            error = ERROR_EJECT;
                        },
                    }
                    self.set_state(state);
                    self.set_error(error);
                }
            },
            FloppyInternalState::WaitToWrite => {
                if self.rw_wait_cycles > cycles {
                    self.rw_wait_cycles -= cycles;
                } else {
                    self.internal_state = FloppyInternalState::Idle;
                    self.rw_wait_cycles = 0;
                    let mut state = self.state();
                    let error;
                    match self.disk {
                        Some(ref mut floppy_disk) => {
                            // Make sure sector exists
                            while floppy_disk.sectors.len() <= (self.rw_sector as usize) {
                                floppy_disk.sectors.push([0; FLOPPY_SECTOR_SIZE]);
                            }
                            match floppy_disk.sectors.get_mut(self.rw_sector as usize) {
                                Some(ref mut s) => {
                                    for i in 0..FLOPPY_SECTOR_SIZE {
                                        s[i as usize] = cpu.mem[self.rw_dcpu_address.wrapping_add(i as u16) as usize];
                                    }
                                },
                                None => {
                                    unreachable!();
                                },
                            }

                            // TODO: Potentially do a sector clean-up if trailing sectors
                            // are all-zero

                            state = match floppy_disk.write_protected {
                                true => STATE_READY_WP,
                                false => STATE_READY,
                            };
                            error = ERROR_NONE;
                        },
                        None => {
                            error = ERROR_EJECT;
                        },
                    }
                    self.set_state(state);
                    self.set_error(error);
                }
            },
            FloppyInternalState::Idle => {},
        }

        if self.interrupt_queued {
            if self.interrupt_message != 0 {
                cpu.interrupt(self.interrupt_message);
            }
            self.interrupt_queued = false;
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
