mod args;
mod errors;
mod matrix;
mod types;
mod data_sources;

use args::command;
use errors::MyError;
use log::{debug, error, info, log_enabled, trace};
use matrix::types::Matrix;
use std::env::set_var;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), MyError> {
    let matches = command().get_matches();

    let debug = matches.get_flag("debug");
    set_var(
        "RUST_LOG",
        match debug {
            true => "debug",
            false => "off",
        },
    );
    env_logger::init();
    println!("test:{}", debug);
    debug!("in debug mode");

    let matrix: Matrix;

    if let Some(matrix_file) = matches.get_one::<String>("matrix") {
        info!("matrix file:{}", matrix_file.to_string());
        matrix = Matrix::from_excel_file(&matrix_file)?;
    } else {
        return Err(MyError::ArgInputError("data source".to_owned()));
    }

    let filter = matches.get_one::<String>("filter");
    // TODO: filter parse

    // parse data input
    // match (
    //     matches.get_one::<String>("input_from_adb"),
    //     matches.get_one::<String>("input_from_file"),
    //     matches.get_one::<String>("input_from_local_interface"),
    // ) {
    //     (None, None, None) => todo!(),
    //     (None, None, Some(input_from_local_interface)) => {
    //         info!(
    //             "input_from_local_interface:{}",
    //             input_from_local_interface.to_string()
    //         );
    //         todo!()
    //     }
    //     (None, Some(input_from_file), None) => {
    //         info!("input_from_file:{}", input_from_file);
    //         todo!()
    //     }
    //     (Some(input_from_adb), None, None) => {
    //         info!("input_from_adb:{}", input_from_adb);
    //         todo!()
    //     }
    //     _ => {
    //         return Err(MyError::ArgInputError("data source".to_owned()));
    //     }
    // }

    // Ok((matrix,))
    Ok(())
}
