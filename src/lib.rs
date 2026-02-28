//! A compact asynchronous `WaitGroup` synchronization primitive.
//!
//! This crate is designed to be lightweight and executor-agnostic. It works
//! with any `async` runtime and supports `no_std` environments (requires
//! `alloc`).
//!
//! # Usage
//!
//! ## [`MonoWaitGroup`]
//!
//! ```rust
//! # use compact_waitgroup::MonoWaitGroup;
//! # futures_executor::block_on(async {
//! let (wg, token) = MonoWaitGroup::new();
//! assert!(!wg.is_done());
//! std::thread::spawn(move || {
//!     // Long-running task
//!     token.release();
//! });
//! // Wait for the task to complete
//! wg.await;
//! # });
//! ```
//!
//! ## [`WaitGroup`]
//!
//! ```rust
//! # use compact_waitgroup::WaitGroup;
//! # futures_executor::block_on(async {
//! let (wg, factory) = WaitGroup::new();
//! factory.scope(|token| {
//!     let token_cloned = token.clone();
//!     assert!(!wg.is_done());
//!     std::thread::spawn(move || {
//!         // Long-running task
//!         token_cloned.release();
//!     });
//!     std::thread::spawn(move || {
//!         // Another long-running task
//!         token.release();
//!     });
//! });
//! // Wait for all tasks to complete
//! wg.await;
//! # });
//! ```
//!
//! ## With `async` Runtime
//!
//! ```rust
//! # use core::{iter::repeat_n, mem::drop as spawn, time::Duration};
//! # use compact_waitgroup::{GroupTokenExt, WaitGroup};
//! # let sleep = |_| async {};
//! # futures_executor::block_on(async {
//! let (wg, factory) = WaitGroup::new();
//! for (i, token) in repeat_n(factory.into_token(), 8).enumerate() {
//!     let task = async move {
//!         println!("Task {i} started");
//!         // Long-running task...
//!         sleep(Duration::from_secs(1)).await;
//!         println!("Task {i} finished");
//!     }
//!     .release_on_ready(token);
//!     spawn(task);
//! }
//! // Wait for all tasks to complete
//! wg.await;
//! # });
//! ```
//!
//! # Memory Layout
//!
//! This crate is designed to be extremely lightweight. The memory footprint
//! depends on the architecture and the enabled features.
//!
//! By default, [`MonoWaitGroup`] shares the same underlying memory structure as
//! [`WaitGroup`]. However, this means [`MonoWaitGroup`] carries a `usize` field
//! for reference counting of workers, which is redundant for the singly-owned
//! [`MonoGroupToken`].
//!
//! Enabling the `compact-mono` feature changes the internal definition of
//! [`MonoWaitGroup`]. It switches to a dedicated, stripped-down layout that
//! removes the reference counter.
//!
//! | Component           | Default (64-bit) | With `compact-mono` | Saving      |
//! | ------------------- | ---------------- | ------------------- | ----------- |
//! | **`WaitGroup`**     | 32 bytes         | 32 bytes            | 0 bytes     |
//! | **`MonoWaitGroup`** | **32 bytes**     | **24 bytes**        | **8 bytes** |
#![no_std]
extern crate alloc;

mod ext;
mod group;
mod layout;
mod sync;
mod twin_ref;
mod utils;

pub use crate::{
    ext::{GroupTokenExt, GroupTokenReleaseOnDrop, GroupTokenReleaseOnReady},
    group::{GroupToken, MonoGroupToken, MonoWaitGroup, WaitGroup},
};

#[cfg(test)]
mod tests;
