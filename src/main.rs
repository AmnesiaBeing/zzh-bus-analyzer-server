// mod someipmatrixloader;
// mod someiptypes;
mod args;
mod matrix;
mod types;

use args::get_args;
use log::{error, info};
use std::env::set_var;
use std::time::Instant;

fn main() {
    set_var("RUST_LOG", "debug");
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
    let now = Instant::now();
    let (matrix_file_path, debug_mode, interface, filter) = get_args()?;

    info!("Task completed in {:?}", now.elapsed());

    Ok(())
}
