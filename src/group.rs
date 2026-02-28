use core::{
    pin::Pin,
    task::{Context, Poll},
};

use derive_more::{Debug, Into};

use crate::{
    layout::SharedLayout,
    sync::{WaitGroupLayoutExt, WaitGroupWrapper},
    twin_ref::{ClonableTwinRef, TwinRef},
};

#[cfg(feature = "compact-mono")]
type MonoLayout = crate::layout::MonoLayout;
#[cfg(not(feature = "compact-mono"))]
type MonoLayout = crate::layout::SharedLayout;

/// WaitGroup with clonable group tokens.
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
/// let (wg, token) = WaitGroup::new();
/// let mut wg = core::pin::pin!(wg);
///
/// assert!(!wg.is_done());
///
/// token.release();
///
/// wg.as_mut().await;
/// assert!(wg.is_done());
///
/// // It is safe to await again (re-poll)
/// wg.as_mut().await;
/// assert!(wg.is_done());
/// # });
/// ```
#[must_use]
#[derive(Debug)]
pub struct WaitGroup(#[debug("done: {}", _0.is_done())] WaitGroupWrapper<TwinRef<SharedLayout>>);

/// WaitGroup with a single non-clonable group token.
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
/// let (wg, token) = MonoWaitGroup::new();
/// let mut wg = core::pin::pin!(wg);
///
/// assert!(!wg.is_done());
///
/// token.release();
///
/// wg.as_mut().await;
/// assert!(wg.is_done());
///
/// // It is safe to await again (re-poll)
/// wg.as_mut().await;
/// assert!(wg.is_done());
/// # });
/// ```
#[must_use]
#[derive(Debug)]
pub struct MonoWaitGroup(#[debug("done: {}", _0.is_done())] WaitGroupWrapper<TwinRef<MonoLayout>>);

/// Clonable group token.
#[must_use]
#[derive(Clone, Debug)]
pub struct GroupToken(
    #[allow(unused)]
    #[debug("done: {}", _0.is_done())]
    ClonableTwinRef<SharedLayout>,
);

/// Non-clonable group token.
#[must_use]
#[derive(Debug)]
pub struct MonoGroupToken(#[debug("done: {}", _0.is_done())] TwinRef<MonoLayout>);

/// Factory of `GroupToken`.
#[must_use]
#[derive(Debug, Into)]
pub struct GroupTokenFactory(GroupToken);

impl WaitGroup {
    /// Creates a new `WaitGroup` and a `GroupTokenFactory`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::WaitGroup;
    ///
    /// let (wg, factory) = WaitGroup::new();
    /// // ... distribute token with factory ...
    /// ```
    pub fn new() -> (Self, GroupTokenFactory) {
        let inner = SharedLayout::new();
        let (wg, token) = TwinRef::new_clonable(inner);
        (
            Self(WaitGroupWrapper::new(wg)),
            GroupTokenFactory(GroupToken(token)),
        )
    }

    /// Checks if the `WaitGroup` has completed.
    ///
    /// This returns `true` if all `GroupToken`s have been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::WaitGroup;
    ///
    /// let (wg, token) = WaitGroup::new();
    /// assert!(!wg.is_done());
    ///
    /// drop(token);
    /// assert!(wg.is_done());
    /// ```
    #[inline]
    pub fn is_done(&self) -> bool {
        self.0.is_done()
    }
}

impl MonoWaitGroup {
    /// Creates a new `MonoWaitGroup` and a single `MonoGroupToken`.
    ///
    /// This variant is optimized for scenarios where there is exactly one
    /// worker task. The token cannot be cloned.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::MonoWaitGroup;
    ///
    /// let (wg, token) = MonoWaitGroup::new();
    /// ```
    pub fn new() -> (Self, MonoGroupToken) {
        let inner = MonoLayout::new();
        let (wg, token) = TwinRef::new_mono(inner);
        (Self(WaitGroupWrapper::new(wg)), MonoGroupToken(token))
    }

    /// Checks if the `MonoWaitGroup` has completed.
    ///
    /// This returns `true` if the `MonoGroupToken` has been dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_waitgroup::MonoWaitGroup;
    ///
    /// let (wg, token) = MonoWaitGroup::new();
    /// assert!(!wg.is_done());
    ///
    /// drop(token);
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

impl GroupTokenFactory {
    /// Consumes the inner token.
    ///
    /// This is equivalent to dropping the factory.
    #[inline]
    pub fn release(self) {
        drop(self);
    }

    #[inline]
    pub fn into_token(self) -> GroupToken {
        self.0
    }

    #[inline]
    pub fn scope<T, F: FnOnce(GroupToken) -> T>(self, func: F) -> T {
        func(self.into_token())
    }
}

impl GroupToken {
    /// Consumes the token.
    ///
    /// This is equivalent to dropping the token.
    #[inline]
    pub fn release(self) {
        drop(self);
    }
}

impl MonoGroupToken {
    /// Consumes the token.
    ///
    /// This is equivalent to dropping the token.
    #[inline]
    pub fn release(self) {
        drop(self);
    }

    #[inline]
    pub fn into_token(self) -> Self {
        self
    }

    #[inline]
    pub fn scope<T, F: FnOnce(MonoGroupToken) -> T>(self, func: F) -> T {
        func(self.into_token())
    }
}

impl Drop for MonoGroupToken {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.0.send_done();
        }
    }
}
