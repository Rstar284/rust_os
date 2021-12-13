use core::sync::atomic::{AtomicU64, Ordering};

pub mod thread;
pub mod thread_switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u64);

impl ThreadId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    fn new(&self) -> Self {
        static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed))
    }
}