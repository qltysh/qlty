use cap::Cap;
use std::alloc;

#[global_allocator]
pub static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);
