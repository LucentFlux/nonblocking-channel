use std::{
    mem::MaybeUninit,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use ringbuf::{Consumer, HeapRb, Producer, SharedRb};

/// The result of trying to send a message.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SendResult<T> {
    Ok,
    Full(T),
    Disconnected,
}

impl<T> SendResult<T> {
    /// Checks if the receiving end of the channel had been dropped when a message was sent.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }

    /// Checks if the message was sent.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}
impl<T: std::fmt::Debug> SendResult<T> {
    pub fn unwrap(self) {
        match self {
            SendResult::Ok => return,
            SendResult::Full(message) => panic!(
                "failed to send message - queue was full when sending {:?}",
                message
            ),
            SendResult::Disconnected => panic!("client was disconnected when sending message"),
        }
    }
}

/// A SPSC sender guaranteed to never block when sending a message. This is a strong constraint, enforced
/// by WebAssembly on the main thread, so this should only be preferred over other mpsc channels where
/// non-blocking behaviour is *required*.
///
/// Cannot be cloned, so if you want multiple clients to send messages then you need a sender that may block
/// for very short periods when sending a message - see [`NonBlockingSender::mpsc`].
pub struct NonBlockingSender<T> {
    inner: Producer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>,
    is_closed: Arc<AtomicBool>,
}
impl<T> NonBlockingSender<T> {
    /// Tries to send a message to the receiving channel without ever blocking, even briefly.
    ///
    /// # Result
    ///
    /// This method fails if the receiving queue is full, or if the receiver has been dropped.
    pub fn try_send(&mut self, message: T) -> SendResult<T> {
        if self.is_closed.load(Ordering::SeqCst) {
            SendResult::Disconnected
        } else {
            let res = self.inner.push(message);
            if let Err(message) = res {
                SendResult::Full(message)
            } else {
                SendResult::Ok
            }
        }
    }

    /// Makes this channel into a `mpsc` channel, but where the sender is allowed to block for very small durations.
    pub fn mpsc(self) -> MicroBlockingSender<T> {
        return MicroBlockingSender {
            inner: Arc::new(Mutex::new(self)),
        };
    }
}

impl<T> Drop for NonBlockingSender<T> {
    fn drop(&mut self) {
        // Cascade closures
        self.is_closed.store(true, Ordering::SeqCst);
    }
}

/// A variant of a [`NonBlockingSender`] which can block but only for very short durations of time, much like the
/// channel in `std` or `flume`. Holding this variant of sender does not mean that the receiver will ever block
/// when receiving - the receiver provided by this crate will never block.
pub struct MicroBlockingSender<T> {
    inner: Arc<Mutex<NonBlockingSender<T>>>,
}

impl<T> MicroBlockingSender<T> {
    /// Adds a state change to the set of changes to enact before the next frame.
    /// This operation is submitted to the GPU only when `RenderData::submit_changes` is called on the main thread.
    ///
    /// This method blocks, but only briefly, and so should not be used on the main thread of a web application.
    pub fn try_send(&self, message: T) -> SendResult<T> {
        self.inner.lock().unwrap().try_send(message)
    }
}

impl<T> Clone for MicroBlockingSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// The result of trying to receive a message.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecvResult<T> {
    Ok(T),
    Empty,
    Disconnected,
}

impl<T> RecvResult<T> {
    /// Checks if the sending end(s) of the channel had been dropped when a message was sent.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }

    /// Checks if either a message was received, or there was nothing in the channel.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_) | Self::Empty)
    }
}
impl<T> RecvResult<T> {
    pub fn unwrap(self) -> Option<T> {
        match self {
            RecvResult::Ok(message) => Some(message),
            RecvResult::Empty => None,
            RecvResult::Disconnected => panic!("receiver was disconnected when receiving message"),
        }
    }
}

/// A receiver that is guaranteed to never block when receiving messages.
pub struct NonBlockingReceiver<T> {
    inner: Consumer<T, Arc<SharedRb<T, Vec<MaybeUninit<T>>>>>,
    is_closed: Arc<AtomicBool>,
}
impl<T> NonBlockingReceiver<T> {
    /// Tries to send a message to the receiving channel without ever blocking, even briefly.
    ///
    /// # Result
    ///
    /// This method fails if the receiving queue is full, or if the receiver has been dropped.
    pub fn try_recv(&mut self) -> RecvResult<T> {
        if self.is_closed.load(Ordering::SeqCst) {
            RecvResult::Disconnected
        } else {
            let res = self.inner.pop();
            if let Some(message) = res {
                RecvResult::Ok(message)
            } else {
                RecvResult::Empty
            }
        }
    }
}

impl<T> Drop for NonBlockingReceiver<T> {
    fn drop(&mut self) {
        // Cascade closures
        self.is_closed.store(true, Ordering::SeqCst);
    }
}

pub fn nonblocking_channel<T>(
    capacity: NonZeroUsize,
) -> (NonBlockingSender<T>, NonBlockingReceiver<T>) {
    let (sender, receiver) = HeapRb::<T>::new(capacity.get()).split();
    let is_closed = Arc::new(AtomicBool::from(false));

    let sender = NonBlockingSender {
        inner: sender,
        is_closed: Arc::clone(&is_closed),
    };
    let receiver = NonBlockingReceiver {
        inner: receiver,
        is_closed,
    };

    return (sender, receiver);
}
