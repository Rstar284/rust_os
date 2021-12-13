use x86_64::VirtAddr;
use crate::memory::StackBounds;
use super::ThreadId;

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    stack_pointer: Option<VirtAddr>,
    stack_bound: Option<StackBounds>,
}