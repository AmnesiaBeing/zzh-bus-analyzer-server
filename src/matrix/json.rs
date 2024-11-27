use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use log::{debug, info};

use crate::errors::MyError;

use super::types::Matrix;

impl Matrix {
    pub fn from_json_file<P>(path: P) -> Result<Matrix, MyError>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        match serde_json::from_slice(&buffer) {
            Ok(ret) => Ok(ret),
            Err(_) => Err(MyError::ParseMatrixFileError(
                "parse json matrix file error.".to_string(),
            )),
        }
    }

    pub fn to_json_file<P>(&self, path: P) -> Result<(), MyError>
    where
        P: AsRef<Path>,
    {
        let mut file = File::create(path)?;

        let data = serde_json::to_string(&self).unwrap();

        file.write_all(data.as_bytes())?;

        file.sync_all()?;

        Ok(())
    }
}

#[test]
fn test() -> Result<(), MyError> {
    use std::env;
    use std::env::set_var;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::path::Path;

    set_var("RUST_LOG", "debug");
    env_logger::init();

    let matrix = Matrix::from_excel_file("./matrix.xlsx").expect("error file");

    let tmp_dir = env::temp_dir();
    let tmp_file_path = tmp_dir.join("temp_data.txt").canonicalize().unwrap();
    debug!("write json matrix file to {}.", tmp_file_path.to_string_lossy());

    matrix.to_json_file(tmp_file_path)
}
