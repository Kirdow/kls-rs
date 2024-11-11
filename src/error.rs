use std::fmt;
use std::error::Error;

#[derive(Debug)]
#[allow(dead_code)]
pub enum KlsError {
    S(String),
    E(String, Box<dyn Error>)
}

impl fmt::Display for KlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S(msg) => write!(f, "KlsError: {}", msg),
            Self::E(msg, e) => write!(f, "KlsError({}): {}", e, msg)
        }
    }
}

impl Error for KlsError {}

