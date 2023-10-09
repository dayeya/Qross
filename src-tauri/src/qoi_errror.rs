use std::io;
use std::fmt;

pub enum QoiError {
    InvalidHeader(String),
    InvalidEndMark(String),
    SavingError(String),
    GeneralIOError(std::io::Error),
}

impl From<io::Error> for QoiError {
    fn from(error: std::io::Error) -> Self {
        QoiError::GeneralIOError(error)
    }
}

impl fmt::Display for QoiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QoiError::InvalidHeader(err) => write!(f, "Invalid MAGICheader error: {}", err),
            QoiError::InvalidEndMark(err) => write!(f, "Invalid end mark error: {}", err),
            QoiError::SavingError(err) => write!(f, "Saving buffer into QOI file resulted an error: {}", err),
            QoiError::GeneralIOError(err) => write!(f, "General io error: {}", err),
        }
    }
}

impl fmt::Debug for QoiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QoiError::InvalidHeader(err) => write!(f, "Invalid header error: {}", err),
            QoiError::InvalidEndMark(err) => write!(f, "Invalid end mark error: {}", err),
            QoiError::SavingError(err) => write!(f, "Saving buffer into QOI file resulted an error: {}", err),
            QoiError::GeneralIOError(err) => write!(f, "General io error: {}", err),
        }
    }
}
