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
//! let (wg, handle) = MonoWaitGroup::new();
//! assert!(!wg.is_done());
//! std::thread::spawn(move || {
//!     // Long-running task
//!     handle.done();
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
//! let (wg, handle) = WaitGroup::new();
//! let handle_cloned = handle.clone();
//! assert!(!wg.is_done());
//! std::thread::spawn(move || {
//!     // Long-running task
//!     handle_cloned.done();
//! });
//! std::thread::spawn(move || {
//!     // Another long-running task
//!     handle.done();
//! });
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
//! [`MonoWorkerHandle`].
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

mod core_impl;
mod state;
mod twin_ref;
mod utils;
mod wait_group;

pub use crate::wait_group::{MonoWaitGroup, MonoWorkerHandle, WaitGroup, WorkerHandle};

#[cfg(test)]
mod tests;
