use core::{
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};

use derive_more::Into;
use pin_project_lite::pin_project;

use crate::{GroupToken, MonoGroupToken};

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

impl<F, H> WithWorkerHandleFuture<F, H> {
    pub fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut F> {
        self.project().inner
    }

    pub fn worker_handle(&self) -> &H {
        &self.worker_handle
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

impl WorkerHandleType for GroupToken {}
impl WorkerHandleType for MonoGroupToken {}
