// ---------------------------------------------------------------------
// Gufo Ping: Module definition
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use pyo3::prelude::*;
pub(crate) mod error;
pub(crate) use error::{PingError, PingResult};
pub(crate) mod filter;
pub(crate) mod session;
pub(crate) use session::SessionManager;
pub(crate) mod proto;
pub(crate) use proto::{
    PS_DGRAM, PS_DGRAM_RAW, PS_IPV4, PS_IPV6, PS_RAW, PS_RAW_DGRAM, Probe, Proto, SelectionPolicy,
};
pub(crate) mod slice;
pub(crate) mod socket;
pub(crate) use socket::SocketWrapper;
pub(crate) mod timer;
pub(crate) use timer::Timer;

/// Module index
#[pymodule]
#[pyo3(name = "_fast")]
fn gufo_ping(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SocketWrapper>()?;
    m.add("PS_DGRAM", PS_DGRAM)?;
    m.add("PS_DGRAM_RAW", PS_DGRAM_RAW)?;
    m.add("PS_IPV4", PS_IPV4)?;
    m.add("PS_IPV6", PS_IPV6)?;
    m.add("PS_RAW", PS_RAW)?;
    m.add("PS_RAW_DGRAM", PS_RAW_DGRAM)?;
    Ok(())
}
