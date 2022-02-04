#[macro_use]
extern crate clap;

use crate::copy_directions::from_string_list;
use crate::executor::execute;

mod copy_directions;
mod executor;
mod file_ops;

fn main() -> std::io::Result<()> {
    let matches = clap_app!(backup_manager =>
        (version: "0.1.0")
        (author: "Chris N. <christopher.neely1200@gmail.com>")
        (about: "Quick file copier")
        (@arg COMMAND: --command -c +takes_value +required "Command in format 'FROM:TO:[skip_ext[,...]]'")
    )
    .get_matches();

    let files = matches
        .values_of("COMMAND")
        .unwrap()
        .into_iter()
        .map(|v| v.as_bytes().to_vec())
        .collect();
    let command_list = from_string_list(files).unwrap();
    execute(command_list)?;

    Ok(())
}
