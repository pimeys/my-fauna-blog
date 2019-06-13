#[derive(Debug)]
pub enum Error {
    Conversion,
    Other,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ERROR HAPPENED")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "ERROR ERROR"
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
