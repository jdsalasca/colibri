//! Minimal single-threaded reactive primitives for experimental research.
//! Automatic synchronous dependencies and batching live in `reactive`.
//! Browser rendering, async tracking and hydration are still unimplemented.
mod reactive;
pub use reactive::{
    batch, untrack, use_effect, use_effect_with_cleanup, use_memo, use_state, Effect, Memo,
};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

type Watcher = Rc<dyn Fn()>;

struct Inner<T> {
    value: RefCell<T>,
    watchers: RefCell<BTreeMap<usize, Watcher>>,
    next_id: Cell<usize>,
}

/// A value with explicitly registered observers.
/// Notifications run synchronously after mutation.
pub struct Signal<T> {
    inner: Rc<Inner<T>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(Inner {
                value: RefCell::new(value),
                watchers: RefCell::new(BTreeMap::new()),
                next_id: Cell::new(0),
            }),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        reactive::track_signal(self);
        self.peek(f)
    }

    /// Read without tracking this signal as a dependency.
    pub fn peek<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.inner.value.borrow())
    }

    pub(crate) fn identity(&self) -> usize {
        Rc::as_ptr(&self.inner) as usize
    }

    #[cfg(test)]
    pub(crate) fn subscriber_count(&self) -> usize {
        self.inner.watchers.borrow().len()
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    pub fn set(&self, value: T) {
        self.update(|current| *current = value);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut current = self.inner.value.borrow_mut();
            f(&mut current);
        }
        // Hold reactive flush until all direct subscribers are notified.
        let _notification = reactive::notification_guard();
        // Snapshot observers, allowing subscriptions to change in a callback.
        let watchers: Vec<Watcher> = self.inner.watchers.borrow().values().cloned().collect();
        for watcher in watchers {
            watcher();
        }
    }

    pub fn subscribe(&self, callback: impl Fn() + 'static) -> Subscription<T> {
        let id = self.inner.next_id.get();
        self.inner
            .next_id
            .set(id.checked_add(1).expect("subscriber id exhausted"));
        self.inner
            .watchers
            .borrow_mut()
            .insert(id, Rc::new(callback));
        Subscription {
            inner: Rc::downgrade(&self.inner),
            id,
        }
    }
}

pub struct Subscription<T> {
    inner: Weak<Inner<T>>,
    id: usize,
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            self.inner = Weak::new();
            inner.watchers.borrow_mut().remove(&self.id);
        }
    }
}

/// Cleanup callbacks run in reverse order when the scope is dropped.
#[derive(Default)]
pub struct Scope {
    cleanups: Vec<Box<dyn FnOnce()>>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_cleanup(&mut self, callback: impl FnOnce() + 'static) {
        self.cleanups.push(Box::new(callback));
    }

    pub fn track<T: 'static>(&mut self, subscription: Subscription<T>) {
        self.on_cleanup(move || drop(subscription));
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        while let Some(callback) = self.cleanups.pop() {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriptions_are_notified_and_removed() {
        let signal = Signal::new(1);
        let calls = Rc::new(Cell::new(0));
        let counter = Rc::clone(&calls);
        let subscription = signal.subscribe(move || counter.set(counter.get() + 1));
        signal.update(|value| *value += 2);
        assert_eq!(signal.get(), 3);
        assert_eq!(calls.get(), 1);
        drop(subscription);
        signal.set(4);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn scope_cleans_subscriptions() {
        let signal = Signal::new(0);
        let calls = Rc::new(Cell::new(0));
        {
            let mut scope = Scope::new();
            let counter = Rc::clone(&calls);
            scope.track(signal.subscribe(move || counter.set(counter.get() + 1)));
            signal.set(1);
        }
        signal.set(2);
        assert_eq!(calls.get(), 1);
    }
}
