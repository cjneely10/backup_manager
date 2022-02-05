#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;

use crate::copy_directions::from_string_list;
use crate::executor::execute;

mod copy_directions;
mod executor;
mod file_ops;
mod test_utils;

#[cfg(not(tarpaulin_include))]
fn main() {
    let matches = clap_app!(backup_manager =>
        (version: "0.1.0")
        (author: "Chris N. <christopher.neely1200@gmail.com>")
        (about: "Quick file copier")
        (@arg INPUT_FILE: -i --input_file +takes_value +required "File with command lines in format 'FROM:TO:[skip_ext[,...]]'")
        (@arg VERBOSE: -v --verbose "Display files copied")
    )
    .get_matches();

    let verbose = matches.is_present("VERBOSE");
    let file = matches.value_of("INPUT_FILE").unwrap();
    let file = File::open(file);
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Unable to parse file!");
            eprintln!("{:?}", e);
            exit(1);
        }
    };
    let reader = BufReader::new(file);
    let command_strings = reader
        .lines()
        .map(|l| l.unwrap())
        .map(|v| v.as_bytes().to_vec())
        .filter(|v| !v.is_empty())
        .collect();
    let command_list = match from_string_list(command_strings) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.message);
            exit(2);
        }
    };
    match execute(command_list, verbose) {
        Ok(summary) => {
            println!("{:?}", summary);
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
}
