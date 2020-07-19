#[derive(Debug, PartialEq)]
pub struct CustomError(pub String);

impl From<std::io::Error> for CustomError {
    fn from(e: std::io::Error) -> Self {
        CustomError(e.to_string())
    }
}

impl From<liquid::Error> for CustomError {
    fn from(e: liquid::Error) -> Self {
        CustomError(e.to_string())
    }
}
