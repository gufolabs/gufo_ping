// ---------------------------------------------------------------------
// Gufo Ping: Session implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-23, Gufo Labs
// ---------------------------------------------------------------------

use std::cmp::Ordering;

/// Ping probe state
/// sid is a string of <addr>-<request id>-<seq>
/// deeadline - is timeout deadline in nanoseconds
/// according to Socket::get_ts()
#[derive(PartialEq, Eq, Clone)]
pub(crate) struct Session {
    sid: String,
    deadline: u64,
}

impl Session {
    /// Create new session
    pub fn new(sid: &str, deadline: u64) -> Self {
        Session {
            sid: sid.to_string(),
            deadline,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self, ts: u64) -> bool {
        self.deadline < ts
    }

    /// Get owned instance of sid
    pub fn get_sid(&self) -> String {
        self.sid.clone()
    }
}

impl Ord for Session {
    /// Sorting for BTreeSet.
    /// Sorting order - (deadline, sid)
    fn cmp(&self, other: &Self) -> Ordering {
        match self.deadline.cmp(&other.deadline) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.sid.cmp(&other.sid),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for Session {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
