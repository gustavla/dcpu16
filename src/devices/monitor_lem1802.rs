use dcpu::{self, DCPU, Device};
use std::any::Any;

pub struct DeviceMonitorLEM1802 {
    pub connected: bool,
    pub ram_location: u16,
    pub font_location: Option<u16>,
    pub palette_location: Option<u16>,
    pub border_color_index: u16,
}

pub const MONITOR_WIDTH: u32 = COLS as u32 * FONT_WIDTH as u32;
pub const MONITOR_HEIGHT: u32 = ROWS as u32 * FONT_HEIGHT as u32;
pub const ROWS: usize = 12;
pub const COLS: usize = 32;
pub const FONT_WIDTH: usize = 4;
pub const FONT_HEIGHT: usize = 8;
pub const SCALE: u32 = 5;
pub const BORDER: u32 = 5*SCALE;

const DEFAULT_FONT: &'static [u16] = &[
    0xb79e, 0x388e, 0x722c, 0x75f4, 0x19bb, 0x7f8f, 0x85f9, 0xb158, 0x242e, 0x2400, 0x082a, 0x0800,
    0x0008, 0x0000, 0x0808, 0x0808, 0x00ff, 0x0000, 0x00f8, 0x0808, 0x08f8, 0x0000, 0x080f, 0x0000,
    0x000f, 0x0808, 0x00ff, 0x0808, 0x08f8, 0x0808, 0x08ff, 0x0000, 0x080f, 0x0808, 0x08ff, 0x0808,
    0x6633, 0x99cc, 0x9933, 0x66cc, 0xfef8, 0xe080, 0x7f1f, 0x0701, 0x0107, 0x1f7f, 0x80e0, 0xf8fe,
    0x5500, 0xaa00, 0x55aa, 0x55aa, 0xffaa, 0xff55, 0x0f0f, 0x0f0f, 0xf0f0, 0xf0f0, 0x0000, 0xffff,
    0xffff, 0x0000, 0xffff, 0xffff, 0x0000, 0x0000, 0x005f, 0x0000, 0x0300, 0x0300, 0x3e14, 0x3e00,
    0x266b, 0x3200, 0x611c, 0x4300, 0x3629, 0x7650, 0x0002, 0x0100, 0x1c22, 0x4100, 0x4122, 0x1c00,
    0x1408, 0x1400, 0x081c, 0x0800, 0x4020, 0x0000, 0x0808, 0x0800, 0x0040, 0x0000, 0x601c, 0x0300,
    0x3e49, 0x3e00, 0x427f, 0x4000, 0x6259, 0x4600, 0x2249, 0x3600, 0x0f08, 0x7f00, 0x2745, 0x3900,
    0x3e49, 0x3200, 0x6119, 0x0700, 0x3649, 0x3600, 0x2649, 0x3e00, 0x0024, 0x0000, 0x4024, 0x0000,
    0x0814, 0x2241, 0x1414, 0x1400, 0x4122, 0x1408, 0x0259, 0x0600, 0x3e59, 0x5e00, 0x7e09, 0x7e00,
    0x7f49, 0x3600, 0x3e41, 0x2200, 0x7f41, 0x3e00, 0x7f49, 0x4100, 0x7f09, 0x0100, 0x3e41, 0x7a00,
    0x7f08, 0x7f00, 0x417f, 0x4100, 0x2040, 0x3f00, 0x7f08, 0x7700, 0x7f40, 0x4000, 0x7f06, 0x7f00,
    0x7f01, 0x7e00, 0x3e41, 0x3e00, 0x7f09, 0x0600, 0x3e41, 0xbe00, 0x7f09, 0x7600, 0x2649, 0x3200,
    0x017f, 0x0100, 0x3f40, 0x3f00, 0x1f60, 0x1f00, 0x7f30, 0x7f00, 0x7708, 0x7700, 0x0778, 0x0700,
    0x7149, 0x4700, 0x007f, 0x4100, 0x031c, 0x6000, 0x0041, 0x7f00, 0x0201, 0x0200, 0x8080, 0x8000,
    0x0001, 0x0200, 0x2454, 0x7800, 0x7f44, 0x3800, 0x3844, 0x2800, 0x3844, 0x7f00, 0x3854, 0x5800,
    0x087e, 0x0900, 0x4854, 0x3c00, 0x7f04, 0x7800, 0x447d, 0x4000, 0x2040, 0x3d00, 0x7f10, 0x6c00,
    0x417f, 0x4000, 0x7c18, 0x7c00, 0x7c04, 0x7800, 0x3844, 0x3800, 0x7c14, 0x0800, 0x0814, 0x7c00,
    0x7c04, 0x0800, 0x4854, 0x2400, 0x043e, 0x4400, 0x3c40, 0x7c00, 0x1c60, 0x1c00, 0x7c30, 0x7c00,
    0x6c10, 0x6c00, 0x4c50, 0x3c00, 0x6454, 0x4c00, 0x0836, 0x4100, 0x0077, 0x0000, 0x4136, 0x0800,
    0x0201, 0x0201, 0x0205, 0x0200
];

const DEFAULT_PALETTE: &'static [u16] = &[
    0x0000, 0x000a, 0x00a0, 0x00aa, 0x0a00, 0x0a0a, 0x0a50, 0x0aaa,
    0x0555, 0x055f, 0x05f5, 0x05ff, 0x0f55, 0x0f5f, 0x0ff5, 0x0fff,
];

// Logo pixels, describes row-major values where pixel is yellow
const LOGO_PIXELS: &'static [usize] = &[
    3379, 3389, 3507, 3517, 3636, 3645, 3764, 3773, 3893, 3901, 4014,
    4021, 4029, 4142, 4150, 4157, 4270, 4271, 4278, 4285, 4398, 4399,
    4407, 4413, 4422, 4423, 4424, 4425, 4426, 4427, 4428, 4429, 4430,
    4431, 4432, 4433, 4526, 4528, 4535, 4541, 4654, 4656, 4664, 4669,
    4782, 4785, 4792, 4797, 4910, 4913, 4921, 4925, 5038, 5042, 5049,
    5053, 5166, 5170, 5178, 5181, 5294, 5299, 5306, 5309, 5422, 5427,
    5435, 5437, 5550, 5556, 5563, 5565, 5678, 5684, 5692, 5693, 5702,
    5703, 5704, 5705, 5706, 5707, 5708, 5709, 5710, 5711, 5712, 5713,
    5806, 5813, 5820, 5821, 5934, 5941, 5949, 6062, 6070, 6077, 6190,
    6198, 6318, 6327, 6446, 6455, 6574, 6584, 6702, 6712, 7462, 7463,
    7466, 7468, 7471, 7472, 7477, 7478, 7479, 7481, 7485, 7486, 7487,
    7489, 7491, 7493, 7494, 7495, 7497, 7498, 7499, 7501, 7504, 7505,
    7507, 7509, 7512, 7513, 7590, 7592, 7594, 7595, 7596, 7598, 7599,
    7600, 7605, 7606, 7609, 7613, 7614, 7617, 7618, 7622, 7625, 7626,
    7629, 7632, 7635, 7636, 7639, 7640, 7641, 7718, 7720, 7723, 7726,
    7728, 7733, 7734, 7735, 7737, 7738, 7739, 7741, 7742, 7743, 7745,
    7747, 7750, 7753, 7755, 7757, 7759, 7760, 7763, 7765, 7767, 7769,
];

impl DeviceMonitorLEM1802 {
    pub fn new() -> DeviceMonitorLEM1802 {
        DeviceMonitorLEM1802 {
            connected: false,
            ram_location: 0,
            font_location: None,
            palette_location: None,
            border_color_index: 0,
        }
    }

    pub fn with_pre_connect(self, location: u16) -> DeviceMonitorLEM1802 {
        let mut new_self = self;
        new_self.connected = true;
        new_self.ram_location = location;
        new_self
    }

    pub fn with_font_location(self, location: u16) -> DeviceMonitorLEM1802 {
        let mut new_self = self;
        new_self.font_location = Some(location);
        new_self
    }

    /*
    pub fn with_palette_location(self, location: u16) -> DeviceMonitorLEM1802 {
        let mut new_self = self;
        new_self.palette_location = Some(location);
        new_self
    }

    pub fn with_border_color_index(self, border_color_index: u16) -> DeviceMonitorLEM1802 {
        let mut new_self = self;
        new_self.border_color_index = border_color_index & 0xf;
        new_self
    }
    */

    pub fn get_font_character(&self, cpu: &DCPU, c: usize) -> u32 {
        match self.font_location {
            Some(loc) => {
                ((cpu.mem[(loc as usize + c * 2) % dcpu::MEMORY_SIZE] as u32) << 16) +
                 (cpu.mem[(loc as usize + c * 2 + 1) % dcpu::MEMORY_SIZE] as u32)
            },
            None => {
                ((DEFAULT_FONT[c * 2] as u32) << 16) +
                 (DEFAULT_FONT[c * 2 + 1] as u32)
            }
        }
    }

    pub fn get_color(&self, cpu: &DCPU, color_index: u16) -> u16 {
        match self.palette_location {
            Some(loc) => {
                cpu.mem[loc.wrapping_add(color_index) as usize]
            },
            None => {
                DEFAULT_PALETTE[color_index as usize]
            },
        }
    }

    pub fn get_border_color_rgb(&self, cpu: &DCPU) -> (u8, u8, u8) {
        if self.connected {
            let color = self.get_color(cpu, self.border_color_index);
            (
                ((((color >> 8) & 0xf) << 4) | ((color >> 8) & 0xf)) as u8,
                ((((color >> 4) & 0xf) << 4) | ((color >> 4) & 0xf)) as u8,
                ((((color     ) & 0xf) << 4) | ((color     ) & 0xf)) as u8,
            )
        } else {
            (0, 0, 170)
        }
    }

    pub fn data(&self, cpu: &DCPU, blinkout: bool) -> Vec<u8> {
        let mut v: Vec<u8> = vec![0; (MONITOR_WIDTH * MONITOR_HEIGHT * 3) as usize];
        {
            let mut slice = &mut v[..];
            if self.connected {
                for i in 0..ROWS {
                    for j in 0..COLS {
                        let mem = cpu.mem[(self.ram_location as usize + i * COLS + j) % dcpu::MEMORY_SIZE];
                        let c = (mem & 0x7f) as usize;

                        let blink = ((mem >> 7) & 1) == 1;
                        let bg_color_index = ((mem >> 8) & 0xf) as u16;
                        let fg_color_index = ((mem >> 12) & 0xf) as u16;

                        let bg_color = self.get_color(cpu, bg_color_index);
                        let fg_color = self.get_color(cpu, fg_color_index);

                        let b0 = if !blinkout || !blink {
                            self.get_font_character(cpu, c)
                        } else {
                            0
                        };

                        for x in 0..FONT_WIDTH {
                            for y in 0..FONT_HEIGHT {
                                let p = (b0 >> ((FONT_WIDTH-1-x) * FONT_HEIGHT + y)) & 1;
                                let color = match p {
                                    1 => fg_color,
                                    _ => bg_color,
                                };

                                let index = i * COLS * FONT_WIDTH * FONT_HEIGHT + y * COLS * FONT_WIDTH + j * FONT_WIDTH + x;
                                slice[index * 3    ] = ((((color >> 8) & 0xf) << 4) | ((color >> 8) & 0xf)) as u8;
                                slice[index * 3 + 1] = ((((color >> 4) & 0xf) << 4) | ((color >> 4) & 0xf)) as u8;
                                slice[index * 3 + 2] = ((((color     ) & 0xf) << 4) | ((color     ) & 0xf)) as u8;
                            }
                        }
                    }
                }
            } else {
                // Clear screen to blue
                for i in 0..((MONITOR_WIDTH * MONITOR_HEIGHT) as usize) {
                    slice[i * 3    ] = 0;
                    slice[i * 3 + 1] = 0;
                    slice[i * 3 + 2] = 170;
                }

                // Print logo and "Nya Elektriska" in yellow
                for p in LOGO_PIXELS {
                    slice[p * 3    ] = 255;
                    slice[p * 3 + 1] = 255;
                    slice[p * 3 + 2] = 0;
                }
            }
        }
        v
    }
}


impl Device for DeviceMonitorLEM1802 {
    fn info_hardware_id_upper(&self) -> u16 { 0x7349 }
    fn info_hardware_id_lower(&self) -> u16 { 0xf615 }
    fn info_manufacturer_id_upper(&self) -> u16 { 0x1c6c }
    fn info_manufacturer_id_lower(&self) -> u16 { 0x8b36 }
    fn info_version(&self) -> u16 { 0x1802 }

    fn process_interrupt(&mut self, cpu: &mut DCPU) -> () {
        let a = cpu.reg[0];
        let b = cpu.reg[1];
        match a {
            0 => { /* MEM_MAP_SCREEN */
                if b > 0 {
                    self.ram_location = b;
                    self.connected = true;
                } else {
                    self.connected = false;
                }
                // TODO: Disable screen for a 1 second
            },
            1 => { /* MEM_MAP_FONT */
                if b > 0 {
                    self.font_location = Some(b);
                } else {
                    self.font_location = None;
                }
            },
            2 => { /* MEM_MAP_PALETTE */
                if b > 0 {
                    self.palette_location = Some(b);
                } else {
                    self.palette_location = None;
                }
            },
            3 => { /* SET_BORDER_COLOR */
                self.border_color_index = b & 0xf;
            },
            4 => { /* MEM_DUMP_FONT */
                for i in 0..DEFAULT_FONT.len() {
                    cpu.mem[b.wrapping_add(i as u16) as usize] = DEFAULT_FONT[i];
                }
                cpu.halt(256);
            }
            5 => { /* MEM_DUMP_PALETTE */
                for i in 0..DEFAULT_PALETTE.len() {
                    cpu.mem[b.wrapping_add(i as u16) as usize] = DEFAULT_PALETTE[i];
                }
                cpu.halt(16);
            }
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


