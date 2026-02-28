use crate::utils::*;

#[cfg(not(loom))]
pub(super) use alloc::sync::Arc;
#[cfg(loom)]
pub(super) use loom::sync::Arc;

#[cfg(loom)]
pub(super) use super::loom::FutureTestExt;
#[cfg(not(loom))]
pub(super) use futures_test::future::FutureTestExt;

pub(super) struct SharedData(AtomicU8);

impl SharedData {
    pub fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    pub fn load(&self) -> bool {
        self.0.load(atomic::Acquire) != 0
    }

    pub fn store(&self) {
        self.0.store(1, atomic::Release);
    }
}
