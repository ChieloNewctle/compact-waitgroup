use alloc::{boxed::Box, sync::Arc};
use futures_test::future::FutureTestExt;

use crate::{MonoWaitGroup, WaitGroup, utils::*};

struct SharedData(AtomicU8);

impl SharedData {
    fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    fn load(&self) -> bool {
        self.0.load(atomic::Acquire) != 0
    }

    fn store(&self) {
        self.0.store(1, atomic::Release);
    }
}

#[futures_test::test]
async fn test_wg_await_background() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();
    assert!(!inspector.load());
    handle.done();
    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_wg_await_background_twice() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle_a) = WaitGroup::new();
    let handle_b = handle_a.clone();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();
    assert!(!inspector.load());
    handle_a.done();
    for _ in 0..100 {
        assert!(!inspector.load());
    }
    handle_b.done();
    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_wg_await_background_twice_rev() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle_a) = WaitGroup::new();
    let handle_b = handle_a.clone();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();
    assert!(!inspector.load());
    handle_b.done();
    for _ in 0..100 {
        assert!(!inspector.load());
    }
    handle_a.done();
    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_mono_wg_await_background() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = MonoWaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();
    assert!(!inspector.load());
    handle.done();
    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_wg_await() {
    let (wg, handle) = WaitGroup::new();
    handle.done();
    wg.await;
}

#[futures_test::test]
async fn test_wg_await_multiple_repeat_n() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    for handle in core::iter::repeat_n(handle, 100) {
        assert!(!inspector.load());
        handle.done();
    }

    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_wg_await_multiple_repeat_with() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    for handle in core::iter::repeat_with(move || handle.clone()).take(100) {
        assert!(!inspector.load());
        handle.done();
    }

    bg_wg.await;
    assert!(inspector.load());
}

#[futures_test::test]
async fn test_wg_await_pin_multiple_repeat_n() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    for handle in core::iter::repeat_n(handle, 100) {
        assert!(!inspector.load());
        handle.done();
    }

    let mut bg_wg = core::pin::pin!(bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[futures_test::test]
async fn test_wg_await_pin_multiple_repeat_with() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    for handle in core::iter::repeat_with(move || handle.clone()).take(100) {
        assert!(!inspector.load());
        handle.done();
    }

    let mut bg_wg = core::pin::pin!(bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[futures_test::test]
async fn test_wg_await_pin_multiple_threads() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    let handles = core::iter::repeat_n(handle, 8)
        .map(|h| {
            let (wg, handle) = WaitGroup::new();
            async move {
                wg.await;
                h.done();
            }
            .run_in_background();
            handle
        })
        .collect::<Box<[_]>>();

    assert!(!inspector.load());
    drop(handles);

    let mut bg_wg = core::pin::pin!(bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}

#[futures_test::test]
#[cfg(panic = "unwind")]
async fn test_wg_threads_panic() {
    let canary = Arc::new(SharedData::new());
    let inspector = canary.clone();
    let (bg_wg, bg_handle) = MonoWaitGroup::new();
    let (wg, handle) = WaitGroup::new();
    async move {
        wg.await;
        canary.store();
        bg_handle.done();
    }
    .run_in_background();

    assert!(!inspector.load());

    #[allow(unused)]
    let handles = core::iter::repeat_n(handle, 4)
        .map(|h| {
            let (wg, handle) = WaitGroup::new();
            async move {
                wg.await;
                panic!();
                h.done();
            }
            .run_in_background();
            handle
        })
        .collect::<Box<[_]>>();

    assert!(!inspector.load());
    drop(handles);

    let mut bg_wg = core::pin::pin!(bg_wg);
    bg_wg.as_mut().await;
    assert!(inspector.load());
    assert!(bg_wg.is_done());
}
