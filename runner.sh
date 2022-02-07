#!/bin/bash
set -e

# This script is useful for setting up a simple cron-job. This script is meant to live within the top-level directory
# of this repository, but can be called from any location.
# This script assumes that the 'backup_manager' crate has been compiled in release mode

binary="target/release/backup_manager"

function help_menu() {
  echo ""
  echo "Usage: runner.sh -b <bkm-file> -l <log-file> -e <err-file>"
  echo ""
  echo ""
  echo "-h|--help                           Display this help message"
  echo "-b|--bkm-file <FILE>                File with command lines in format"
  echo "                                    'FROM:TO[:skip-pattern[,...]]'"
  echo "-l|--log-file <FILE>                Append log statements to file"
  echo "-e|--err-file <FILE>                Append err statements to file"
  echo ""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -b|--bkm-file)
      bkm="$2"
      [ ! -f "${bkm:?}" ] && echo -e ".bkm file not found" && exit 1
      shift
      shift
      ;;
    -l|--log-file)
      log="$2"
      shift
      shift
      ;;
    -e|--err-file)
      err="$2"
      shift
      shift
      ;;
    -h|--help)
      help_menu
      exit 0
      ;;
    -*)
      echo -e "Error: Unsupported flag $1" >&2
      help_menu
      exit 1
      ;;
    *)
      echo -e "Error: Unsupported argument $1" >&2
      help_menu
      exit 1
      ;;
  esac
done

[[ -z "${log}" || -z "${err}" || -z "${bkm}" ]] && { help_menu ; exit 1 ; }

# Path to binary. Implementation from
# https://stackoverflow.com/questions/59895/how-can-i-get-the-source-directory-of-a-bash-script-from-within-the-script-itsel
script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
bin="${script_dir:?}/${binary:?}"
[[ ! -f "$bin" ]] && { echo "Unable to locate compiled 'backup_manager' program" ; exit 2 ; }
# Datetime
current_date=$(date)
echo "$current_date" >> "${log:?}"
echo "$current_date" >> "${err:?}"
# Run backup and log output/errors
$bin -i "${bkm:?}" >> "$log" 2>> "$err"
# Extra blank line for ease in reading
echo "" >> "$log"
echo "" >> "$err"
