use std::{fs::File, io::{Read, Write}, path::Path};

use super::types::Matrix;

impl Matrix {
    pub fn from_json_file<P>(path: P) -> Result<Matrix, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        Ok(serde_json::from_slice(&buffer)?)
    }

    pub fn to_json_file<P>(&self, path: P) -> Result<(), Box<dyn std::error::Error>>
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
