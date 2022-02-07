# backup_manager

If you're like me, you've lost scores of files over the years due to improperly managing backups.

After the most recent occurrence, I decided to write a very simple backup tool for creating redundant copies
of my file system.

This crate is very early in its development, and I am not sure how much more it will be developed from here.

## Installation

```shell
git clone https://github.com/cjneely10/backup_manager
cd backup_manager
cargo build --release
```

## Example usage

```shell
./runner.sh -b example.bkm -l example.log -e example.err
```

## Usage from binary (without shell script)

```text
backup_manager 0.1.0
Chris N. <christopher.neely1200@gmail.com>
Quick file copier

USAGE:
    backup_manager [FLAGS] --input_file <INPUT_FILE>

FLAGS:
    -v, --verbose    Display file paths as they are copied
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input_file <INPUT_FILE>    File with command lines in format
                                     'FROM:TO[:skip-pattern[,...]]'
```
