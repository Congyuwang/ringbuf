use crate::sync::{Instant, TimeoutIterator};
use core::time::Duration;
use ringbuf::traits::{Consumer, Producer};

pub trait BlockingProducer: Producer {
    type Instant: Instant;

    fn wait_vacant(&self, count: usize, timeout: Option<Duration>) -> bool;

    fn push(&mut self, item: Self::Item, timeout: Option<Duration>) -> Result<(), Self::Item> {
        if self.wait_vacant(1, timeout) {
            assert!(self.try_push(item).is_ok());
            Ok(())
        } else {
            Err(item)
        }
    }

    fn push_iter_all<I: Iterator<Item = Self::Item>>(&mut self, iter: I, timeout: Option<Duration>) -> usize {
        let mut count = 0;
        let mut iter = iter.peekable();
        for timeout in TimeoutIterator::<Self::Instant>::new(timeout) {
            if iter.peek().is_none() {
                break;
            }
            if self.wait_vacant(1, timeout) {
                count += self.push_iter(&mut iter);
            }
        }
        count
    }

    fn push_slice_all(&mut self, mut slice: &[Self::Item], timeout: Option<Duration>) -> usize
    where
        Self::Item: Copy,
    {
        let mut count = 0;
        for timeout in TimeoutIterator::<Self::Instant>::new(timeout) {
            if slice.is_empty() {
                break;
            }
            if self.wait_vacant(1, timeout) {
                let n = self.push_slice(slice);
                slice = &slice[n..];
                count += n;
            }
        }
        count
    }
}

#[macro_export]
macro_rules! delegate_blocking_producer {
    ($type:ty, $ref:expr, $mut:expr) => {
        type Instant = <$type as $crate::traits::BlockingProducer>::Instant;

        #[inline]
        fn wait_vacant(&self, count: usize, timeout: Option<core::time::Duration>) -> bool {
            $ref(self).wait_vacant(count, timeout)
        }

        #[inline]
        fn push(&mut self, item: Self::Item, timeout: Option<core::time::Duration>) -> Result<(), Self::Item> {
            $mut(self).push(item, timeout)
        }

        #[inline]
        fn push_iter_all<I: Iterator<Item = Self::Item>>(&mut self, iter: I, timeout: Option<core::time::Duration>) -> usize {
            $mut(self).push_iter_all(iter, timeout)
        }

        #[inline]
        fn push_slice_all(&mut self, slice: &[Self::Item], timeout: Option<core::time::Duration>) -> usize
        where
            Self::Item: Copy,
        {
            $mut(self).push_slice_all(slice, timeout)
        }
    };
}

pub trait BlockingConsumer: Consumer {
    type Instant: Instant;

    fn wait_occupied(&self, count: usize, timeout: Option<Duration>) -> bool;

    fn pop_wait(&mut self, timeout: Option<Duration>) -> Option<Self::Item> {
        if self.wait_occupied(1, timeout) {
            Some(self.try_pop().unwrap())
        } else {
            None
        }
    }

    fn pop_iter_all(&mut self, timeout: Option<Duration>) -> PopAllIter<'_, Self> {
        unsafe { PopAllIter::new(self, timeout) }
    }

    fn pop_slice_all(&mut self, mut slice: &mut [Self::Item], timeout: Option<Duration>) -> usize
    where
        Self::Item: Copy,
    {
        let mut count = 0;
        for timeout in TimeoutIterator::<Self::Instant>::new(timeout) {
            if slice.is_empty() {
                break;
            }
            if self.wait_occupied(1, timeout) {
                let n = self.pop_slice(slice);
                slice = &mut slice[n..];
                count += n;
            }
        }
        count
    }
}

pub struct PopAllIter<'a, C: BlockingConsumer> {
    pub(crate) target: &'a mut C,
    pub(crate) timeout: TimeoutIterator<C::Instant>,
}

impl<'a, C: BlockingConsumer> PopAllIter<'a, C> {
    pub unsafe fn new(target: &'a mut C, timeout: Option<Duration>) -> Self {
        Self {
            target,
            timeout: TimeoutIterator::new(timeout),
        }
    }
}

impl<'a, C: BlockingConsumer> Iterator for PopAllIter<'a, C> {
    type Item = C::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let timeout = self.timeout.next()?;
        if self.target.wait_occupied(1, timeout) {
            self.target.try_pop()
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! delegate_blocking_consumer {
    ($type:ty, $ref:expr, $mut:expr) => {
        type Instant = <$type as $crate::traits::BlockingConsumer>::Instant;

        #[inline]
        fn wait_occupied(&self, count: usize, timeout: Option<core::time::Duration>) -> bool {
            $ref(self).wait_occupied(count, timeout)
        }

        #[inline]
        fn pop_wait(&mut self, timeout: Option<core::time::Duration>) -> Option<Self::Item> {
            $mut(self).pop_wait(timeout)
        }

        #[inline]
        fn pop_iter_all(&mut self, timeout: Option<core::time::Duration>) -> $crate::traits::PopAllIter<'_, Self> {
            unsafe { $crate::traits::PopAllIter::new(self, timeout) }
        }

        #[inline]
        fn pop_slice_all(&mut self, slice: &mut [Self::Item], timeout: Option<core::time::Duration>) -> usize
        where
            Self::Item: Copy,
        {
            $mut(self).pop_slice_all(slice, timeout)
        }
    };
}
