// CLI helper functions

use getopts::Options;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn print_usage(program: &str, positional: &str, opts: Options, examples: &[&str]) {
    let brief = format!("{} {}", &opts.short_usage(program), positional);
    print!("{}", opts.usage(&brief));
    if !examples.is_empty() {
        println!("");
        println!("Examples:");
        for ex in examples {
            println!("  {} {}", program, ex);
        }
    }
}

pub fn print_version(program: &str) {
    println!("{} {}", program, VERSION);
}
