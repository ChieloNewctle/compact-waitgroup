use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    core_impl::{WaitGroupUtil, WaitGroupWrapper},
    state::SharedWgInner,
    twin_ref::{ClonableTwinRef, TwinRef},
};

#[cfg(feature = "compact-mono")]
type MonoInner = crate::state::MonoWgInner;
#[cfg(not(feature = "compact-mono"))]
type MonoInner = crate::state::SharedWgInner;

/// WaitGroup with clonable worker handles.
///
/// # Cancellation safety
///
/// This future is cancellation safe.
///
/// It is also safe to poll again after completion.
///
/// ```rust
/// # use compact_waitgroup::WaitGroup;
/// # futures_executor::block_on(async {
/// let (wg, handle) = WaitGroup::new();
/// let mut wg = core::pin::pin!(wg);
///
/// assert!(!wg.is_done());
///
/// handle.done();
///
/// wg.as_mut().await;
/// assert!(wg.is_done());
///
/// // It is safe to await again (re-poll)
/// wg.as_mut().await;
/// assert!(wg.is_done());
/// # });
/// ```
#[derive(Debug)]
pub struct WaitGroup(WaitGroupWrapper<TwinRef<SharedWgInner>>);

/// WaitGroup with a single non-clonable worker handle.
///
/// # Cancellation safety
///
/// This future is cancellation safe.
///
/// It is also safe to poll again after completion.
///
/// ```rust
/// # use compact_waitgroup::MonoWaitGroup;
/// # futures_executor::block_on(async {
/// let (wg, handle) = MonoWaitGroup::new();
/// let mut wg = core::pin::pin!(wg);
///
/// assert!(!wg.is_done());
///
/// handle.done();
///
/// wg.as_mut().await;
/// assert!(wg.is_done());
///
/// // It is safe to await again (re-poll)
/// wg.as_mut().await;
/// assert!(wg.is_done());
/// # });
/// ```
#[derive(Debug)]
pub struct MonoWaitGroup(WaitGroupWrapper<TwinRef<MonoInner>>);

/// Clonable worker handle.
#[derive(Clone, Debug)]
pub struct WorkerHandle {
    _handle: ClonableTwinRef<SharedWgInner>,
}

/// Non-clonable worker handle.
#[derive(Debug)]
pub struct MonoWorkerHandle(TwinRef<MonoInner>);

impl WaitGroup {
    /// Creates a new `WaitGroup` and a clonable `WorkerHandle`.
    ///
    /// The `WaitGroup` is used to await the completion of tasks. The
    /// `WorkerHandle` is used to signal task completion.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::WaitGroup;
    ///
    /// let (wg, handle) = WaitGroup::new();
    /// // ... distribute handle ...
    /// ```
    pub fn new() -> (Self, WorkerHandle) {
        let inner = SharedWgInner::new();
        let (wg, handle) = TwinRef::new_clonable(inner);
        (
            Self(WaitGroupWrapper::new(wg)),
            WorkerHandle { _handle: handle },
        )
    }

    /// Checks if the `WaitGroup` has completed.
    ///
    /// This returns `true` if all `WorkerHandle`s have been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::WaitGroup;
    ///
    /// let (wg, handle) = WaitGroup::new();
    /// assert!(!wg.is_done());
    ///
    /// drop(handle);
    /// assert!(wg.is_done());
    /// ```
    #[inline]
    pub fn is_done(&self) -> bool {
        self.0.is_done()
    }
}

impl MonoWaitGroup {
    /// Creates a new `MonoWaitGroup` and a single `MonoWorkerHandle`.
    ///
    /// This variant is optimized for scenarios where there is exactly one
    /// worker task. The handle cannot be cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::MonoWaitGroup;
    ///
    /// let (wg, handle) = MonoWaitGroup::new();
    /// ```
    pub fn new() -> (Self, MonoWorkerHandle) {
        let inner = MonoInner::new();
        let (wg, handle) = TwinRef::new_mono(inner);
        (Self(WaitGroupWrapper::new(wg)), MonoWorkerHandle(handle))
    }

    /// Checks if the `MonoWaitGroup` has completed.
    ///
    /// This returns `true` if the `MonoWorkerHandle` has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::MonoWaitGroup;
    ///
    /// let (wg, handle) = MonoWaitGroup::new();
    /// assert!(!wg.is_done());
    ///
    /// drop(handle);
    /// assert!(wg.is_done());
    /// ```
    #[inline]
    pub fn is_done(&self) -> bool {
        self.0.is_done()
    }
}

impl Future for WaitGroup {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl Future for MonoWaitGroup {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

#[cfg(feature = "futures-core")]
impl futures_core::FusedFuture for WaitGroup {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

#[cfg(feature = "futures-core")]
impl futures_core::FusedFuture for MonoWaitGroup {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

impl WorkerHandle {
    /// Consumes the handle.
    ///
    /// This is equivalent to dropping the handle.
    #[inline]
    pub fn done(self) {
        drop(self);
    }
}

impl MonoWorkerHandle {
    /// Consumes the handle.
    ///
    /// This is equivalent to dropping the handle.
    #[inline]
    pub fn done(self) {
        drop(self);
    }
}

impl Drop for MonoWorkerHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.0.send_done();
        }
    }
}
