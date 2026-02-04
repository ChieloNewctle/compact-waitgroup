use core::{
    borrow::Borrow,
    fmt::Debug,
    panic::{RefUnwindSafe, UnwindSafe},
};

use derive_more::Deref;

use crate::{
    core_impl::{WaitGroupData, WaitGroupType, WaitGroupUtil},
    twin_ref::{ClonableTwinRefType, TwinRef, TwinRefType},
    utils::*,
};

pub(crate) struct MonoWgInner {
    twin_count: AtomicU8,
    state: AtomicU8,
    data: UnsafeCell<WaitGroupData>,
}

#[cfg(not(loom))]
const _: () = {
    assert!(core::mem::size_of::<MonoWgInner>() == core::mem::size_of::<usize>() * 3);
    assert!(core::mem::align_of::<MonoWgInner>() == core::mem::size_of::<usize>());
};

unsafe impl Send for MonoWgInner {}
unsafe impl Sync for MonoWgInner {}
impl UnwindSafe for MonoWgInner {}
impl RefUnwindSafe for MonoWgInner {}

impl MonoWgInner {
    #[cfg(not(loom))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            twin_count: AtomicU8::new(2),
            state: AtomicU8::new(0),
            data: UnsafeCell::new(WaitGroupData::None),
        }
    }

    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            twin_count: AtomicU8::new(2),
            state: AtomicU8::new(0),
            data: UnsafeCell::new(WaitGroupData::None),
        }
    }
}

#[derive(Deref)]
pub(crate) struct SharedWgInner {
    cloned_count: AtomicUsize,
    #[deref]
    inner: MonoWgInner,
}

#[cfg(not(loom))]
const _: () = {
    assert!(core::mem::size_of::<SharedWgInner>() == core::mem::size_of::<usize>() * 4);
    assert!(core::mem::align_of::<SharedWgInner>() == core::mem::size_of::<usize>());
};

impl SharedWgInner {
    #[cfg(not(loom))]
    #[inline]
    pub const fn new() -> Self {
        Self {
            cloned_count: AtomicUsize::new(1),
            inner: MonoWgInner::new(),
        }
    }

    #[cfg(loom)]
    pub fn new() -> Self {
        Self {
            cloned_count: AtomicUsize::new(1),
            inner: MonoWgInner::new(),
        }
    }
}

impl Debug for MonoWgInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WaitGroupInner")
            .field("done", &self.is_done())
            .finish()
    }
}

impl Debug for SharedWgInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharedWaitGroupInner")
            .field("done", &self.is_done())
            .finish()
    }
}

impl Borrow<MonoWgInner> for SharedWgInner {
    #[inline]
    fn borrow(&self) -> &MonoWgInner {
        self
    }
}

impl Borrow<MonoWgInner> for TwinRef<SharedWgInner> {
    #[inline]
    fn borrow(&self) -> &MonoWgInner {
        self
    }
}

unsafe impl<T: Borrow<MonoWgInner>> TwinRefType for T {
    #[inline]
    fn count(&self) -> &AtomicU8 {
        &self.borrow().twin_count
    }
}

unsafe impl<T: Borrow<MonoWgInner>> WaitGroupType for T {
    #[inline]
    fn state(&self) -> &AtomicU8 {
        &self.borrow().state
    }

    #[inline]
    unsafe fn slot(&self) -> &UnsafeCell<WaitGroupData> {
        &self.borrow().data
    }
}

unsafe impl<T: Borrow<SharedWgInner>> ClonableTwinRefType for T {
    #[inline]
    fn cloned_count(&self) -> &AtomicUsize {
        &self.borrow().cloned_count
    }

    #[inline]
    fn action_on_zero(&self) {
        unsafe {
            self.borrow().send_done();
        }
    }
}
