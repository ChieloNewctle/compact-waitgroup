#![cfg(not(loom))]

use core::panic::{RefUnwindSafe, UnwindSafe};

use static_assertions::{assert_impl_all, assert_not_impl_any};

use crate::{GroupToken, MonoGroupToken, MonoWaitGroup, WaitGroup};

assert_impl_all!(WaitGroup: Sync, Send, UnwindSafe, RefUnwindSafe);
assert_impl_all!(GroupToken: Sync, Send, UnwindSafe, RefUnwindSafe, Clone);
assert_impl_all!(MonoWaitGroup: Sync, Send, UnwindSafe, RefUnwindSafe);
assert_impl_all!(MonoGroupToken: Sync, Send, UnwindSafe, RefUnwindSafe);

assert_not_impl_any!(WaitGroup: Clone);
assert_not_impl_any!(MonoGroupToken: Clone);
assert_not_impl_any!(MonoWaitGroup: Clone);
