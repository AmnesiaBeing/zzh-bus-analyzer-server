use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    Io(#[from] std::io::Error),
    // TODO: convert xlsx error to myerror
    Xlsx(#[from] calamine::XlsxError),
    De(#[from] calamine::DeError),
    ArgInputError(String),
    ParseExcelMatrixFileError(String),
    Custom(String)
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
