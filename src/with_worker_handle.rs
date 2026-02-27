use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use derive_more::Into;
use pin_project_lite::pin_project;

use crate::{MonoWorkerHandle, WorkerHandle};

pin_project! {
    #[derive(Debug, Into)]
    pub struct WithWorkerHandleFuture<F, H> {
        #[pin]
        inner: F,
        worker_handle: H,
    }
}

impl<F, H> Deref for WithWorkerHandleFuture<F, H> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<F, H> DerefMut for WithWorkerHandleFuture<F, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithWorkerHandle<H>: Sized {
    fn with_worker_handle(self, handle: H) -> WithWorkerHandleFuture<Self, H>;
}

impl<F: Future, H: WorkerHandleType> WithWorkerHandle<H> for F {
    fn with_worker_handle(self, handle: H) -> WithWorkerHandleFuture<Self, H> {
        WithWorkerHandleFuture {
            inner: self,
            worker_handle: handle,
        }
    }
}

impl<F: Future, H: WorkerHandleType> Future for WithWorkerHandleFuture<F, H> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}

trait WorkerHandleType {}

impl WorkerHandleType for WorkerHandle {}
impl WorkerHandleType for MonoWorkerHandle {}
