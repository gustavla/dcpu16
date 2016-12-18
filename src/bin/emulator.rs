extern crate dcpu16;
extern crate getopts;

mod cli;

use std::vec::Vec;
use std::path::Path;
use std::{env, thread, time};
use dcpu16::dcpu;
use dcpu16::disassembler;
//use dcpu16::bin::cli;
use getopts::Options;
use std::process::exit;

use dcpu16::devices::clock_generic::DeviceClockGeneric;

const FPS: usize = 30;

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    opts.optflag("p", "print", "print CPU info each tick");
    opts.optflag("v", "version", "print version");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(why) => {
            println!("{}", why);
            exit(1);
        },
    };

    if matches.opt_present("h") {
        cli::print_usage(&program, "FILE", opts, &["-p output.bin"]);
        return;
    }

    if matches.opt_present("v") {
        cli::print_version(&program);
        return;
    }

    if matches.free.len() != 1 {
        println!("Please input file");
        return;
    }
    let print = matches.opt_present("p");
    let ref filename = matches.free[0];

    let mut cpu = dcpu::DCPU::new();

    let path = Path::new(filename);
    match cpu.load_from_binary_file(&path) {
        Ok(()) => {},
        Err(why) => {
            println!("Could load file {}: {}", path.display(), why);
            exit(1);
        },
    }

    // Connect hardware
    //cpu.devices.push(Box::new(dcpu::HWMonitorLEM1802{connected: false, ram_location: 0}));
    /*
    let mut floppy = Box::new(dcpu::HWFloppyM35FD::new());
    floppy.sectors.push(dcpu::HWFloppyM35FDSector{mem: [1; 512]});
    floppy.state = 1;
    cpu.devices.push(floppy);
    */

    let clock = DeviceClockGeneric::new();
    cpu.add_device(Box::new(clock));

    // If printing is turned on, CPU will tick through (without proper timing)
    if print {
        while !cpu.terminate {
            cpu.tick();
            let (_, s) = disassembler::disassemble_instruction(&cpu, true);
            println!("---------------------------------------------");
            println!("::: {}", s);
            cpu.print();
        }
    } else { // If printing is not on, then the CPU will run roughly at 100 kHz
        let cycles = dcpu::CYCLE_HZ / FPS;
        while !cpu.terminate {
            //let now = time::Instant::now();
            cpu.run(cycles);
            //let elapsed = now.elapsed();
            // TODO: Use elapsed to sleep slightly shorter to get timing right
            thread::sleep(time::Duration::from_millis((1000 / FPS) as u64));
        }
    }
}
