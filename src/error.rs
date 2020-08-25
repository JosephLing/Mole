use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum CustomError{
    IOError(String),
    LiquidError(String)
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::IOError(s) => write!(f, "IO error: {}\n", s),
            CustomError::LiquidError(s) => write!(f, "Liquid error: {}\n", s),
        }
    }
}

impl From<std::io::Error> for CustomError {
    fn from(e: std::io::Error) -> Self {
        CustomError::IOError(e.to_string())
    }
}

impl From<liquid::Error> for CustomError {
    fn from(e: liquid::Error) -> Self {
        CustomError::LiquidError(e.to_string())
    }
}
