use clap::builder::NonEmptyStringValueParser;
use clap::{crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, Command};
use log::{error, info};
use std::error::Error;
use std::path::Path;

fn command() -> Command {
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

pub fn get_args() -> Result<(String, bool, String, String),String> {
    let matches = command().get_matches();

    let matrix_file = matches
        .get_one::<String>("matrix");
    if matrix_file.is_none(){
        return Err("arg matrix file not found!".to_string());
    }
    let matrix_file=matrix_file.unwrap();
    if !Path::new(&matrix_file).is_file() {
        return Err("arg matrix file not found!".to_string());
    }
    info!("matrix file:{}",matrix_file.to_string());

    let debug = matches.get_flag("debug");
    info!("debug:{}",debug);

    let filter = matches.get_one::<String>("filter");

    let input_from_adb = matches.get_one::<String>("input_from_adb");
    let input_from_file = matches.get_one::<String>("input_from_file");
    let input_from_local_interface = matches.get_one::<String>("input_from_local_interface");

    if input_from_adb.is_some() {
        // TODO: dump form android device
        return Err("dump from android device will be supported.".to_string());
    }

    if input_from_local_interface.is_some() {
        info!("input_from_local_interface:{}",input_from_local_interface.unwrap().trim().to_string());
        return Ok((matrix_file.to_string(),debug,input_from_local_interface.unwrap().trim().to_string(),filter.unwrap().trim().to_string()));
    }

    if input_from_file.is_some() {
        let input_from_file = input_from_file.unwrap();
        if !Path::new(&input_from_file).is_file() {
            return Err("arg input pcap file not found!".to_string());
        }
        return Ok((matrix_file.to_string(),debug,input_from_local_interface.unwrap().trim().to_string(),filter.unwrap().trim().to_string()));
    }

    Err("Invalid Arg".to_string())
}

#[cfg(test)]
mod args_tests {
    use crate::args::command;

    #[test]
    fn verify_command() {
        command().debug_assert();
    }
}
