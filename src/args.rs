use clap::builder::NonEmptyStringValueParser;
use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, Command};
use log::{error, info};
use std::error::Error;
use std::path::Path;


pub fn command() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::new("matrix")
                .help("the matrix file, json or xlsx.")
                .value_parser(NonEmptyStringValueParser::new())
                .long("matrix")
                .short('m')
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("matrix-excel")
                .help("load the excel matrix file.")
                .value_parser(NonEmptyStringValueParser::new())
                .long("matrix-excel")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("matrix-json")
                .help("lod the json matrix file.")
                .value_parser(NonEmptyStringValueParser::new())
                .long("matrix-json")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("matrix-excel-to-json")
                .help("convert the excel matrix file to json file.")
                .value_parser(NonEmptyStringValueParser::new())
                .long("matrix-excel-to-json")
                .short('c')
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("debug")
                .help("debug info")
                .long("debug")
                .short('d')
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("input_from_file")
                .help("parse from pcap file")
                .long("file")
                .value_parser(NonEmptyStringValueParser::new())
                .short('f')
                .num_args(1),
        )
        .arg(
            Arg::new("input_from_local_interface")
                .help("parse from net interface, link enp3s0.")
                .long("interface")
                .value_parser(NonEmptyStringValueParser::new())
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("input_from_adb")
                .help("parse from android devices, allow specify an interface.")
                .value_parser(NonEmptyStringValueParser::new())
                .long("adb")
                .num_args(1),
        )
        .group(
            clap::ArgGroup::new("input")
                .args(&["input_from_file", "input_from_local_interface", "input_from_adb"])
                .required(true)
                .multiple(false)
        )
        // TODO: filter expression allow complex expression
        .arg(
            Arg::new("filter")
                .help("filter expression, like: (serviceid).(methodid) or (servicename).(methodname).(datatypename).(datatypename).[time,subscribe,value,trend]")
                .last(true)
                .required(true)                
        )
}

#[cfg(test)]
mod args_tests {
    use crate::args::command;

    #[test]
    fn verify_command() {
        command().debug_assert();
    }
}
