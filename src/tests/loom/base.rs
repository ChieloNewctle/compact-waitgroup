use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_test::task::new_count_waker;

use crate::{MonoWaitGroup, WaitGroup};

#[test]
fn test_wg_done() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle) = WaitGroup::new();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 1);
    });
}

#[test]
fn test_wg_done_twice() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle_a) = WaitGroup::new();
        let handle_b = handle_a.clone();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle_a.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle_b.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 1);
    });
}

#[test]
fn test_wg_done_twice_rev() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle_a) = WaitGroup::new();
        let handle_b = handle_a.clone();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle_b.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle_a.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 1);
    })
}

#[test]
fn test_mono_wg_done() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle) = MonoWaitGroup::new();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Pending);
        handle.release();
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 1);
    })
}

#[test]
fn test_wg_send_before_poll() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle) = WaitGroup::new();
        handle.release();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 0);
    })
}

#[test]
fn test_mono_wg_send_before_poll() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);
        let (wg, handle) = MonoWaitGroup::new();
        handle.release();
        let mut rx = core::pin::pin!(wg);
        assert_eq!(rx.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(counter.get(), 0);
    })
}

#[test]
fn test_wg_drop_before_send() {
    loom::model(|| {
        let (wg, handle) = WaitGroup::new();
        drop(wg);
        handle.release();
    })
}

#[test]
fn test_mono_wg_drop_before_send() {
    loom::model(|| {
        let (wg, handle) = MonoWaitGroup::new();
        drop(wg);
        handle.release();
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_wg_panic_both() {
    loom::model(|| {
        let (_wg, _handle) = WaitGroup::new();
        panic!();
        drop((_wg, _handle));
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_wg_panic_wg() {
    loom::model(|| {
        let (_wg, handle) = WaitGroup::new();
        drop(handle);
        panic!();
        drop(_wg);
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_wg_panic_handle() {
    loom::model(|| {
        let (wg, _handle) = WaitGroup::new();
        drop(wg);
        panic!();
        drop(_handle);
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_mono_wg_panic_both() {
    loom::model(|| {
        let (_wg, _handle) = MonoWaitGroup::new();
        panic!();
        drop((_wg, _handle));
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_mono_wg_panic_wg() {
    loom::model(|| {
        let (_wg, handle) = MonoWaitGroup::new();
        drop(handle);
        panic!();
        drop(_wg);
    })
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
fn test_mono_wg_panic_handle() {
    loom::model(|| {
        let (wg, _handle) = MonoWaitGroup::new();
        drop(wg);
        panic!();
        drop(_handle);
    })
}

#[test]
fn test_wg_poll_by_others() {
    loom::model(|| {
        let (waker_a, counter_a) = new_count_waker();
        let (waker_b, counter_b) = new_count_waker();

        let (wg, handle) = WaitGroup::new();
        let mut wg = core::pin::pin!(wg);

        let mut cx = Context::from_waker(&waker_a);
        assert_eq!(wg.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(counter_a.get(), 0);

        let mut cx = Context::from_waker(&waker_b);
        assert_eq!(wg.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(counter_a.get(), 0);
        assert_eq!(counter_b.get(), 0);

        handle.release();

        assert_eq!(counter_a.get(), 0);
        assert_eq!(counter_b.get(), 1);

        assert_eq!(wg.as_mut().poll(&mut cx), Poll::Ready(()));
        assert_eq!(wg.as_mut().poll(&mut cx), Poll::Ready(()));

        assert_eq!(counter_a.get(), 0);
        assert_eq!(counter_b.get(), 1);
    });
}

#[test]
fn test_wg_drop_early() {
    loom::model(|| {
        let (waker, counter) = new_count_waker();
        let mut cx = Context::from_waker(&waker);

        let (mut wg, handle) = WaitGroup::new();
        let pinned_wg = Pin::new(&mut wg);
        assert_eq!(pinned_wg.poll(&mut cx), Poll::Pending);

        drop(wg);

        handle.release();
        assert_eq!(counter.get(), 0);
    });
}
