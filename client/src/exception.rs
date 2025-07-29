/// The exceptions that can occur loading resources
#[derive(Debug)]
pub enum Exception {
    /// An io error occurred
    IoError(std::io::Error),
    ContentError,
}

impl From<std::io::Error> for Exception {
    fn from(a: std::io::Error) -> Self {
        Exception::IoError(a)
    }
}
