#![feature(slicing_syntax)] 

extern crate getopts;
extern crate rustdoc;

use getopts::{optopt, optflag, getopts, usage};
use rustdoc::html::highlight::highlight;
use std::io::fs::File;
use std::io::{FileMode, FileAccess};
use std::os;

fn main() {
    let args: Vec<String> = os::args();

    let opts = &[
        optopt("i", "inputfile", "use a file for the input", "FILE"),
        optopt("o", "outfile", "use a file for the output", "FILE"),
        optflag("h", "help", "print this help menu")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        println!("{}", usage("", opts));
        return;
    }

    let input =  match matches.opt_str("i"){
        Some(file) => {
            match File::open(&Path::new(file)).read_to_string() {
                Ok(src) => src,
                Err(e) => panic!(e)
            }
        },
        None => {
            if matches.free.len() == 0 {
                "".to_string()
            } else {
                matches.free[0].to_string()
            }
        }
    };

    let html = highlight(input[], None, None);

    match matches.opt_str("o"){
        Some(file) => {
            let mut out_file = File::open_mode(&Path::new(file), FileMode::Open, FileAccess::Write);
            match out_file.write_str(html[]) {
                Ok(_) => {},
                Err(e) => { panic!(e); }
            };
        },
        None => {
            println!("{}", html);
        }
    };
}