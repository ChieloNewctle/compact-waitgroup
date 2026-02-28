use alloc::boxed::Box;
use loom::sync::Arc;

use crate::{MonoWaitGroup, WaitGroup, WithWorkerHandle, utils::*};

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

trait WaitInPlace {
    fn wait_in_place(self);
}

impl<T: Future> WaitInPlace for T {
    fn wait_in_place(self) {
        loom::future::block_on(self);
    }
}

trait RunInBackground {
    fn run_in_background(self);
}

impl<T: Future + 'static> RunInBackground for T {
    fn run_in_background(self) {
        loom::thread::spawn(move || {
            self.wait_in_place();
        });
    }
}

#[test]
fn test_wg_await_background() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();
        assert!(!inspector.load());
        handle.release();
        bg_wg.wait_in_place();
        assert!(inspector.load());
    });
}

#[test]
fn test_wg_await_background_twice() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle_a) = WaitGroup::new();
        let handle_b = handle_a.clone();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();
        assert!(!inspector.load());
        handle_a.release();
        for _ in 0..100 {
            assert!(!inspector.load());
        }
        handle_b.release();
        bg_wg.wait_in_place();
        assert!(inspector.load());
    })
}

#[test]
fn test_wg_await_background_twice_rev() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle_a) = WaitGroup::new();
        let handle_b = handle_a.clone();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();
        assert!(!inspector.load());
        handle_b.release();
        for _ in 0..100 {
            assert!(!inspector.load());
        }
        handle_a.release();
        bg_wg.wait_in_place();
        assert!(inspector.load());
    })
}

#[test]
fn test_mono_wg_await_background() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = MonoWaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();
        assert!(!inspector.load());
        handle.release();
        bg_wg.wait_in_place();
        assert!(inspector.load());
    })
}

#[test]
fn test_wg_await() {
    loom::model(|| {
        let (wg, handle) = WaitGroup::new();
        handle.release();
        wg.wait_in_place();
    })
}

#[test]
fn test_wg_await_multiple_repeat_n() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();

        assert!(!inspector.load());

        for handle in core::iter::repeat_n(handle, 100) {
            assert!(!inspector.load());
            handle.release();
        }

        bg_wg.wait_in_place();
        assert!(inspector.load());
    })
}

#[test]
fn test_wg_await_multiple_repeat_with() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();

        assert!(!inspector.load());

        for handle in core::iter::repeat_with(move || handle.clone()).take(100) {
            assert!(!inspector.load());
            handle.release();
        }

        bg_wg.wait_in_place();
        assert!(inspector.load());
    })
}

#[test]
fn test_wg_await_pin_multiple_repeat_n() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();

        assert!(!inspector.load());

        for handle in core::iter::repeat_n(handle, 100) {
            assert!(!inspector.load());
            handle.release();
        }

        async move {
            let mut bg_wg = core::pin::pin!(bg_wg);
            bg_wg.as_mut().await;
            assert!(inspector.load());
            assert!(bg_wg.is_done());
        }
        .wait_in_place();
    })
}

#[test]
fn test_wg_await_pin_multiple_repeat_with() {
    loom::model(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();

        assert!(!inspector.load());

        for handle in core::iter::repeat_with(move || handle.clone()).take(100) {
            assert!(!inspector.load());
            handle.release();
        }

        async move {
            let mut bg_wg = core::pin::pin!(bg_wg);
            bg_wg.as_mut().await;
            assert!(inspector.load());
            assert!(bg_wg.is_done());
        }
        .wait_in_place();
    })
}

#[test]
fn test_wg_await_pin_multiple_threads() {
    let mut builder = loom::model::Builder::new();
    builder.preemption_bound = Some(2);
    builder.check(|| {
        let canary = Arc::new(SharedData::new());
        let inspector = canary.clone();
        let (bg_wg, bg_handle) = MonoWaitGroup::new();
        let (wg, handle) = WaitGroup::new();
        async move {
            wg.await;
            canary.store();
        }
        .with_worker_handle(bg_handle)
        .run_in_background();

        assert!(!inspector.load());

        let handles = core::iter::repeat_n(handle, 2)
            .map(|h| {
                let (wg, handle) = WaitGroup::new();
                async move {
                    wg.await;
                }
                .with_worker_handle(h)
                .run_in_background();
                handle
            })
            .collect::<Box<[_]>>();

        assert!(!inspector.load());
        drop(handles);

        async move {
            let mut bg_wg = core::pin::pin!(bg_wg);
            bg_wg.as_mut().await;
            assert!(inspector.load());
            assert!(bg_wg.is_done());
        }
        .wait_in_place();
    });
}
