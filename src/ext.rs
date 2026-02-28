use core::{
    pin::Pin,
    task::{Context, Poll},
};

use derive_more::Into;
use pin_project_lite::pin_project;

use crate::{GroupToken, MonoGroupToken};

pub trait GroupTokenExt<T>: Sized {
    fn release_on_drop(self, token: T) -> GroupTokenReleaseOnDrop<Self, T> {
        GroupTokenReleaseOnDrop { inner: self, token }
    }

    fn release_on_ready(self, token: T) -> GroupTokenReleaseOnReady<Self, T> {
        GroupTokenReleaseOnReady {
            inner: self,
            token: Some(token),
        }
    }
}

trait GroupTokenType {}

impl GroupTokenType for GroupToken {}
impl GroupTokenType for MonoGroupToken {}

impl<T: GroupTokenType, F: Future> GroupTokenExt<T> for F {}

pin_project! {
    #[derive(Debug, Into)]
    pub struct GroupTokenReleaseOnDrop<F, T> {
        #[pin]
        inner: F,
        token: T,
    }
}

pin_project! {
    #[derive(Debug, Into)]
    pub struct GroupTokenReleaseOnReady<F, T> {
        #[pin]
        inner: F,
        token: Option<T>,
    }
}

impl<F, T> GroupTokenReleaseOnDrop<F, T> {
    pub fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut F> {
        self.project().inner
    }

    pub fn group_token(&self) -> &T {
        &self.token
    }
}

impl<F, T> GroupTokenReleaseOnReady<F, T> {
    pub fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut F> {
        self.project().inner
    }

    pub fn group_token(&self) -> Option<&T> {
        self.token.as_ref()
    }
}

impl<F: Future, T> Future for GroupTokenReleaseOnDrop<F, T> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner_pin().poll(cx)
    }
}

impl<F: Future, T> Future for GroupTokenReleaseOnReady<F, T> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = this.inner.poll(cx);
        if res.is_ready() {
            drop(this.token.take());
        }
        res
    }
}
