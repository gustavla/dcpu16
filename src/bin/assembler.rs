extern crate dcpu16;
extern crate getopts;

mod cli;

use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::env;
use getopts::Options;
use dcpu16::assembler;
use std::process::exit;

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    opts.optopt("o", "output", "output binary file to path (otherwise defaults to output.bin)", "PATH");
    opts.optflag("v", "version", "print version");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(why) => {
            println!("{}", why);
            return;
        },
    };

    if matches.opt_present("h") {
        cli::print_usage(&program, "FILE", opts, &["program.asm -o program.bin"]);
        return;
    }

    if matches.opt_present("v") {
        cli::print_version(&program);
        return;
    }

    if matches.free.len() != 1 {
        println!("Please input file");
        exit(1);
    }
    let output_filename = match matches.opt_str("output") {
        Some(s) => s,
        None => "output.bin".to_string(),
    };
    let ref filename = matches.free[0];

    let mut lines: Vec<String> = Vec::new();

    let x: &[_] = &[' ', '\n', '\t'];
    let path = Path::new(filename);
    let file = match File::open(&path) {
        Err(why) => {
            println!("Could not open file {}: {}", path.display(), why.description());
            exit(1);
        },
        Ok(file) => file,
    };
    for line in BufReader::new(&file).lines() {
        match line {
            Ok(s) => {lines.push(s.trim_matches(x).to_string())},
            Err(_) => {},
        }
    }

    let mut cpu = assembler::PCPU::new();
    let ret = assembler::parse(&lines, &mut cpu);

    let mut buf = [0u8; 2];

    match ret {
        Ok(_) => {
            let file = match File::create(&Path::new(&output_filename)) {
                Err(why) => {
                    println!("Could not open file {}: {}", path.display(), why.description());
                    exit(1);
                },
                Ok(f) => f,
            };
            let mut writer = BufWriter::new(&file);
            for i in 0..cpu.pc {
                // DCPU-16 assembled files use little-endian
                let v = cpu.mem[i as usize];
                buf[0] = (v >> 8) as u8;
                buf[1] = (v & 0xff) as u8;
                let ret2 = writer.write(&buf);
                //println!("{}: {:04x}", i, cpu.mem[i as usize]);
                //let ret = file.write_le_u16(cpu.mem[i as usize]);
                match ret2 {
                    Ok(_) => {
                    },
                    Err(_) => {
                        println!("IO Error");
                        break;
                    }
                }

                //io::stdout().write_be_u16(cpu.mem[i as usize]);
            }
        },
        Err(err) => {
            assembler::print_parse_error(&cpu, &lines[err.line as usize][..], err);
            exit(1);
        },
    }

    /*
    assert_eq!("11foo1bar11".trim_matches('1'), "foo1bar");
    let x: &[_] = &['1', '2'];
    assert_eq!("12foo1bar12".trim_matches(x), "foo1bar");
    assert_eq!("123foo1bar123".trim_matches(|&: c: char| c.is_numeric()), "foo1bar");
    */

    //io::stdout().write(&buf);
}
