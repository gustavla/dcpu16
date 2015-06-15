extern crate dcpu16;
extern crate getopts;

use std::env;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::path::Path;
use getopts::Options;
use dcpu16::dcpu;
use dcpu16::disassembler;

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();
    opts.optflag("m", "no-color", "no color");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(why) => {
            println!("{}", why);
            return;
        },
    };

    if matches.free.len() != 1 {
        println!("Please input file");
        return;
    }
    let color = !matches.opt_present("m");
    let ref filename = matches.free[0];

    let mut cpu = dcpu::DCPU::new();

    let path = Path::new(filename);
    let mut file = match File::open(&path) {
        Err(why) => {
            println!("Could not open file {}: {}", path.display(), why.description());
            return;
        },
        Ok(f) => f,
    };

    let mut nwords = 0usize;
    let mut buffer: Vec<u8> = Vec::new();
    let res = file.read_to_end(&mut buffer);
    match res {
        Ok(_) => {
            let mut j = 0;
            let mut sig = 0u16;
            for v in buffer {
                if j % 2 == 0 {
                    sig = v as u16;
                } else {
                    cpu.mem[nwords] = (sig << 8) + (v as u16);
                    nwords += 1;
                }

                j += 1;
            }
        }
        Err(why) => {
            println!("Could not read contents of file: {}", why);
            return;
        },
    }

    loop {
        if cpu.pc as usize > nwords {
            break;
        }
        let (offset, s) = disassembler::disassemble_instruction(&cpu, color);
        cpu.pc += offset;
        println!("{}", s);
    }
}
