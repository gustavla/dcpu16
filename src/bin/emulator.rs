extern crate dcpu16;
extern crate getopts;

use std::vec::Vec;
use std::path::Path;
use std::env;
use dcpu16::dcpu;
use dcpu16::disassembler;
use getopts::Options;

fn main() {
    let mut opts = Options::new();

    let args: Vec<String> = env::args().collect();
    opts.optflag("p", "print", "print");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!(f.to_string()) },
    };

    if matches.free.len() != 1 {
        println!("Please input file");
        return;
    }
    let print = matches.opt_present("p");
    let ref filename = matches.free[0];

    let mut cpu = dcpu::DCPU::new();

    let path = Path::new(filename);
    match cpu.load_from_assembly_file(&path) {
        Ok(()) => {},
        Err(why) => {
            println!("Could load file {}: {}", path.display(), why);
            return;
        },
    }

    loop {
        let (_, s) = disassembler::disassemble_instruction(&cpu, true);
        if print {
            println!("---------------------------------------------");
            println!("::: {}", s);
            cpu.print();
        }
        cpu.tick();
        if cpu.terminate {
            break;
        }
    }
}
