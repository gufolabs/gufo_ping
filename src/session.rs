// ---------------------------------------------------------------------
// Gufo Ping: SessionManager implementation
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use std::{cmp::Ordering, collections::BTreeSet};

/// Ping probe state
/// sid is a string of <addr>-<request id>-<seq>
/// deeadline - is timeout deadline in nanoseconds
/// according to Socket::get_ts()
#[derive(PartialEq, Eq, Clone)]
struct Session {
    sid: u64,
    deadline: u64,
}

pub(crate) struct SessionManager(BTreeSet<Session>);

impl Session {
    /// Create new session
    pub fn new(sid: u64, deadline: u64) -> Self {
        Session { sid, deadline }
    }

    /// Check if session is expired
    pub fn is_expired(&self, ts: u64) -> bool {
        self.deadline < ts
    }

    /// Get owned instance of sid
    pub fn get_sid(&self) -> u64 {
        self.sid
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

impl SessionManager {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn register(&mut self, sid: u64, deadline: u64) {
        self.0.insert(Session::new(sid, deadline));
    }

    pub fn remove(&mut self, sid: u64, deadline: u64) {
        self.0.remove(&Session::new(sid, deadline));
    }

    pub fn drain_expired(&mut self, deadline: u64) -> DrainExpired<'_> {
        DrainExpired {
            mgr: self,
            deadline,
        }
    }
}

pub(crate) struct DrainExpired<'a> {
    mgr: &'a mut SessionManager,
    deadline: u64,
}

impl Iterator for DrainExpired<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.mgr.0.first() {
            Some(first) if first.is_expired(self.deadline) => {
                self.mgr.0.pop_first().map(|s| s.get_sid())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_sid() {
        let sid = 0x1234;
        let ts = 0x5678;
        let timeout = 1000;
        let session = Session::new(sid, ts + timeout);
        assert_eq!(session.get_sid(), sid)
    }
    #[test]
    fn test_is_expired() {
        let sid = 0x1234;
        let ts = 0x5678;
        let timeout = 1000;
        let session = Session::new(sid, ts + timeout);
        assert!(!session.is_expired(ts));
        assert!(!session.is_expired(ts + timeout / 2));
        assert!(!session.is_expired(ts + timeout));
        assert!(session.is_expired(ts + timeout + 1));
    }

    #[test]
    fn test_cmp1() {
        assert!(Session::new(2, 1) < Session::new(1, 2))
    }
    #[test]
    fn test_cmp2() {
        assert!(Session::new(1, 1) < Session::new(2, 1))
    }
    #[test]
    fn test_cmp3() {
        assert!(Session::new(1, 1) == Session::new(1, 1))
    }
    #[test]
    fn test_cmp4() {
        assert!(Session::new(2, 2) > Session::new(1, 1))
    }
    #[test]
    fn test_register() {
        let sid = 0x1234;
        let ts = 0x5678;
        let timeout = 1;
        let mut sessions = SessionManager::new();
        assert_eq!(sessions.0.len(), 0);
        sessions.register(sid, ts + timeout);
        assert_eq!(sessions.0.len(), 1);
        let r = sessions.0.first();
        assert!(r.is_some());
        assert!(*r.unwrap() == Session::new(sid, ts + timeout));
    }
    #[test]
    fn test_remote_unexistent() {
        let sid = 0x1234;
        let invalid_sid = 0x2345;
        let ts = 0x5678;
        let timeout = 1;
        let mut sessions = SessionManager::new();
        assert_eq!(sessions.0.len(), 0);
        sessions.register(sid, ts + timeout);
        assert_eq!(sessions.0.len(), 1);
        sessions.remove(invalid_sid, ts + timeout);
        assert_eq!(sessions.0.len(), 1); // Not removed
        sessions.remove(sid, ts + timeout / 2);
        assert_eq!(sessions.0.len(), 1); // Not removed
    }
    #[test]
    fn test_remove() {
        let sid = 0x1234;
        let ts = 0x5678;
        let timeout = 1;
        let mut sessions = SessionManager::new();
        assert_eq!(sessions.0.len(), 0);
        sessions.register(sid, ts + timeout);
        assert_eq!(sessions.0.len(), 1);
        sessions.remove(sid, ts + timeout);
        assert_eq!(sessions.0.len(), 0); // removed
    }
    #[test]
    fn test_drain() {
        let start_sid = 0x12345;
        let start_ts = 0x6578;
        let timeout = 5;
        let mut sessions = SessionManager::new();
        for i in 0..10 {
            sessions.register(start_sid + i, start_ts + i + timeout);
        }
        assert_eq!(sessions.0.len(), 10);
        let drained = Vec::from_iter(sessions.drain_expired(start_ts + timeout + 5));
        assert_eq!(
            drained,
            vec![
                start_sid,
                start_sid + 1,
                start_sid + 2,
                start_sid + 3,
                start_sid + 4
            ]
        );
        assert_eq!(sessions.0.len(), 5);
    }
}
