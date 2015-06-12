extern crate dcpu16;
extern crate getopts;

use std::vec::Vec;
use std::path::Path;
use std::fs::File;
use std::io::Read;
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
    let mut file = match File::open(&path) {
        Err(_) => panic!(""),//Could not open file {}: {}", path.display(), Exception::description(&why)),
        Ok(file) => file,
    };
    let mut i = 0;
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
                    cpu.mem[i] = (sig << 8) + (v as u16);
                    i += 1;
                }

                j += 1;
            }
        }
        Err(_) => {
            panic!("Could not read contents of file");
        }
    }

    //let input = io::stdin().read_char().ok().expect("Failed to read line");

    /*for i in buf.iter() {
        println!(":{}", i);
    }
    */

    //let mut j = 0;
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

        //j += 1;
    }

    /*
    println!("{}", input);
    */
}
