#[macro_use]
extern crate clap;
#[cfg(test)]
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
mod summary;
mod test_utils;

#[cfg(not(tarpaulin_include))]
fn main() {
    let matches = clap_app!(backup_manager =>
        (version: "0.1.0")
        (author: "Chris N. <christopher.neely1200@gmail.com>")
        (about: "Quick file copier")
        (@arg INPUT_FILE: -i --input_file +takes_value +required "File with command lines in format\n'FROM:TO[:skip-pattern[,...]]'")
        (@arg VERBOSE: -v --verbose "Display file paths as they are copied")
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
        // Skip empty lines
        .filter(|v| !v.is_empty())
        // Skip lines starting with # - comment lines
        .filter(|v| v[0] != b'#')
        .collect();
    let command_list = match from_string_list(command_strings) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.message);
            exit(2);
        }
    };
    println!("{:?}", execute(command_list, verbose));
}
