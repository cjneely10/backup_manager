#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::copy_directions::from_string_list;
use crate::executor::execute;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod copy_directions;
mod executor;
mod file_ops;
mod test_utils;

#[cfg(not(tarpaulin_include))]
fn main() -> std::io::Result<()> {
    let matches = clap_app!(backup_manager =>
        (version: "0.1.0")
        (author: "Chris N. <christopher.neely1200@gmail.com>")
        (about: "Quick file copier")
        (@arg INPUT_FILE: -i --input_file +takes_value +required "File with command lines in format 'FROM:TO:[skip_ext[,...]]'")
    )
    .get_matches();

    let file = matches.value_of("INPUT_FILE").unwrap();
    let reader = BufReader::new(File::open(file).expect("Unable to locate input file!"));
    let command_strings = reader
        .lines()
        .map(|l| l.unwrap())
        .map(|v| v.as_bytes().to_vec())
        .filter(|v| !v.is_empty())
        .collect();
    let command_list = from_string_list(command_strings).unwrap();
    execute(command_list)?;

    Ok(())
}
