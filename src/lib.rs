// ---------------------------------------------------------------------
// Gufo Ping: Module definition
// ---------------------------------------------------------------------
// Copyright (C) 2022-25, Gufo Labs
// ---------------------------------------------------------------------

use pyo3::prelude::*;
pub(crate) mod session;
pub(crate) use session::Session;
pub(crate) mod icmp;
pub(crate) use icmp::IcmpPacket;
pub(crate) mod socket;
pub(crate) use socket::SocketWrapper;

/// Module index
#[pymodule]
#[pyo3(name = "_fast")]
fn gufo_ping(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SocketWrapper>()?;
    Ok(())
}
