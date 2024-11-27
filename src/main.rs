mod args;
mod errors;
mod matrix;
mod parsers;
mod types;

use args::command;
use errors::MyError;
use log::{debug, error, info, log_enabled, trace};
use matrix::types::Matrix;
use std::env::set_var;
use std::ffi::OsStr;
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
        // 支持excel、json后缀名
        match Path::new(matrix_file).canonicalize() {
            Ok(path) => {
                if let Some(ext) = path.extension() {
                    match ext.to_ascii_lowercase().to_str() {
                        Some("xlsx") | Some("xls") => {
                            matrix = Matrix::from_excel_file(path)?
                        },
                        Some("json") => {
                            matrix = Matrix::from_json_file(path)?
                        },
                        _ => {},
                    }
                } else {
                    return Err(MyError::ArgInputError("arg matrix file extension error".to_owned()));
                }
            }
            Err(_) => {
                return Err(MyError::ArgInputError("arg matrix path error".to_owned()));
            }
        }
    } else {
        return Err(MyError::ArgInputError("arg matrix error".to_owned()));
    }

    let filter = matches.get_one::<String>("filter");
    // TODO: filter parse

    // parse data source
    // let source: Source;
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
