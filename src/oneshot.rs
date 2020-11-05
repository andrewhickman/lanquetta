use std::{cell::UnsafeCell, sync::{Arc, Weak}};

use tokio::sync::Semaphore;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        semaphore: Semaphore::new(0),
        value: UnsafeCell::new(None),
    });

    (
        Sender {
            inner: Arc::downgrade(&inner),
        },
        Receiver { inner },
    )
}

pub struct Sender<T> {
    inner: Weak<Inner<T>>,
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    semaphore: Semaphore,
    value: UnsafeCell<Option<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, value: T) {
        if let Some(inner) = self.inner.upgrade() {
            unsafe {
                *inner.value.get() = Some(value);
            }
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.semaphore.add_permits(usize::MAX >> 4);
        }
    }
}

unsafe impl<T: Send + Sync> Send for Inner<T> {}
unsafe impl<T: Send + Sync> Sync for Inner<T> {}

impl<T> Receiver<T> {
    pub async fn borrow<'a>(&'a self) -> Option<&'a T> {
        let _guard = self.inner.semaphore.acquire().await;
        unsafe { &*self.inner.value.get() }.as_ref()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            inner: self.inner.clone()
        }
    }
}
