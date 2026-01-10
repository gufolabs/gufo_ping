// ---------------------------------------------------------------------
// Gufo Ping: Error and result
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------
use pyo3::{
    PyErr,
    exceptions::{PyNotImplementedError, PyOSError, PyPermissionError, PyValueError},
};
use std::io;

pub(crate) type PingResult<T> = Result<T, PingError>;

#[derive(Debug)]
pub enum PingError {
    SocketError(String),
    InvalidAddr,
    InvalidPolicy,
    NotImplemented,
    PermissionDenied,
}

impl From<PingError> for PyErr {
    fn from(value: PingError) -> PyErr {
        match value {
            PingError::SocketError(x) => PyOSError::new_err(x),
            PingError::InvalidAddr => PyValueError::new_err("invalid address"),
            PingError::InvalidPolicy => PyValueError::new_err("invalid policy"),
            PingError::NotImplemented => PyNotImplementedError::new_err("not implemented"),
            PingError::PermissionDenied => PyPermissionError::new_err("permission denied"),
        }
    }
}

impl From<io::Error> for PingError {
    fn from(value: io::Error) -> Self {
        PingError::SocketError(value.to_string())
    }
}

impl From<std::net::AddrParseError> for PingError {
    fn from(_value: std::net::AddrParseError) -> Self {
        PingError::InvalidAddr
    }
}
