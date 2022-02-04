#[macro_use]
extern crate clap;

mod copy_directions;
mod executor;
mod file_ops;

fn main() {
    let matches = clap_app!();
}
