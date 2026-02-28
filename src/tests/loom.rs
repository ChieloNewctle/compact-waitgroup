#![cfg(loom)]

pub(super) trait FutureTestExt {
    fn wait_in_place(self);
    fn run_in_background(self);
}

impl<T: Future + 'static> FutureTestExt for T {
    fn wait_in_place(self) {
        loom::future::block_on(self);
    }

    fn run_in_background(self) {
        loom::thread::spawn(move || {
            self.wait_in_place();
        });
    }
}

// #[should_panic]

macro_rules! loom_test_case {
    (async $fn_name:ident, $builder_modifier:expr) => {
        #[test]
        fn $fn_name() {
            let mut builder = loom::model::Builder::new();
            ($builder_modifier)(&mut builder);
            builder.check(|| {
                $crate::tests::default::$fn_name().wait_in_place();
            });
        }
    };
    (panic $fn_name:ident, $builder_modifier:expr) => {
        #[test]
        #[should_panic]
        fn $fn_name() {
            let mut builder = loom::model::Builder::new();
            ($builder_modifier)(&mut builder);
            builder.check(|| {
                $crate::tests::default::$fn_name();
            });
        }
    };
    ($fn_name:ident, $builder_modifier:expr) => {
        #[test]
        fn $fn_name() {
            let mut builder = loom::model::Builder::new();
            ($builder_modifier)(&mut builder);
            builder.check(|| {
                $crate::tests::default::$fn_name();
            });
        }
    };
    ($token:tt $fn_name:ident) => {
        loom_test_case!($token $fn_name, |_| {});
    };
    ($fn_name:ident) => {
        loom_test_case!($fn_name, |_| {});
    };
}

// base
loom_test_case!(test_mono_wg_done);
loom_test_case!(test_mono_wg_drop_before_send);
loom_test_case!(test_mono_wg_send_before_poll);
loom_test_case!(test_wg_done);
loom_test_case!(test_wg_done_twice);
loom_test_case!(test_wg_done_twice_rev);
loom_test_case!(test_wg_drop_before_send);
loom_test_case!(test_wg_drop_early);
loom_test_case!(test_wg_poll_by_others);
loom_test_case!(test_wg_send_before_poll);

// futures
loom_test_case!(async test_mono_wg_await_background);
loom_test_case!(async test_mono_wg_pinned_drop_in_another_thread);
loom_test_case!(async test_wg_await);
loom_test_case!(async test_wg_await_background);
loom_test_case!(async test_wg_await_background_twice);
loom_test_case!(async test_wg_await_background_twice_rev);
loom_test_case!(async test_wg_await_multiple_repeat_n);
loom_test_case!(async test_wg_await_multiple_repeat_with);
loom_test_case!(async test_wg_await_pin_multiple_repeat_n);
loom_test_case!(async test_wg_await_pin_multiple_repeat_with);
loom_test_case! {
    async test_mono_wg_await_pin_multiple_threads,
    |builder: &mut loom::model::Builder| {
        builder.preemption_bound = Some(2);
    }
}
loom_test_case! {
    async test_wg_await_pin_multiple_threads,
    |builder: &mut loom::model::Builder| {
        builder.preemption_bound = Some(2);
    }
}

// panic
loom_test_case!(panic test_mono_wg_panic_both);
loom_test_case!(panic test_mono_wg_panic_handle);
loom_test_case!(panic test_mono_wg_panic_wg);
loom_test_case!(panic test_wg_panic_both);
loom_test_case!(panic test_wg_panic_handle);
loom_test_case!(panic test_wg_panic_wg);

// twin_ref
loom_test_case!(test_twin_ref_clonable);
loom_test_case!(test_twin_ref_mono);
