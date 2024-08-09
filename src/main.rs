mod args;
mod matrix;
mod types;

use args::command;
use log::{error, info};
use std::env::set_var;
use std::path::Path;
use std::time::Instant;

fn main() {
    env_logger::init();

    std::process::exit(match main_result() {
        Ok(_) => {
            info!("Process exited.");
            0
        }
        Err(err) => {
            error!("{}", err);
            1
        }
    });
}

fn main_result() -> Result<(), String> {
    let matches = command().get_matches();

    let debug = matches.get_flag("debug");
    set_var(
        "RUST_LOG",
        match debug {
            true => "debug",
            false => "off",
        },
    );

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


    // let now = Instant::now();
    // info!("Task completed in {:?}", now.elapsed());

    Ok(())
}
