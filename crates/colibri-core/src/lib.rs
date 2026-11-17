//! Minimal single-threaded reactive primitives for experimental research.
//! Automatic synchronous dependencies and batching live in `reactive`.
//! Browser rendering, async tracking and hydration are still unimplemented.
mod reactive;
pub use reactive::{
    batch, take_error, untrack, use_effect, use_effect_with_cleanup, use_memo, use_state, Effect,
    Memo, ReactiveError,
};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

type Watcher = Rc<dyn Fn()>;

/// Snapshot of the thread-local reactive counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RuntimeStats {
    pub signals: usize,
    pub subscriptions: usize,
    pub observers: usize,
    pub queued: usize,
}

pub fn stats() -> RuntimeStats {
    STATS.with(|stats| RuntimeStats {
        signals: stats.signals.get(),
        subscriptions: stats.subscriptions.get(),
        observers: stats.observers.get(),
        queued: reactive::queued_tasks(),
    })
}

#[derive(Default)]
struct Stats {
    signals: Cell<usize>,
    subscriptions: Cell<usize>,
    observers: Cell<usize>,
}

thread_local! {
    static STATS: Stats = Stats::default();
}

fn increment(cell: &Cell<usize>) {
    cell.set(cell.get() + 1);
}

fn decrement(cell: &Cell<usize>, amount: usize) {
    cell.set(cell.get().saturating_sub(amount));
}

fn signal_created() {
    STATS.with(|stats| increment(&stats.signals));
}

fn signal_dropped(subscriptions: usize) {
    STATS.with(|stats| {
        decrement(&stats.signals, 1);
        decrement(&stats.subscriptions, subscriptions);
    });
}

fn subscription_created() {
    STATS.with(|stats| increment(&stats.subscriptions));
}

fn subscription_dropped() {
    STATS.with(|stats| decrement(&stats.subscriptions, 1));
}

pub(crate) fn observer_created() {
    STATS.with(|stats| increment(&stats.observers));
}

pub(crate) fn observer_dropped() {
    STATS.with(|stats| decrement(&stats.observers, 1));
}

struct Inner<T> {
    value: RefCell<T>,
    watchers: RefCell<BTreeMap<usize, Watcher>>,
    next_id: Cell<usize>,
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        signal_dropped(self.watchers.borrow().len());
    }
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
        signal_created();
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
        // usize::MAX subscriptions are not reachable; saturate instead of panicking.
        self.inner.next_id.set(id.saturating_add(1));
        subscription_created();
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
            if inner.watchers.borrow_mut().remove(&self.id).is_some() {
                subscription_dropped();
            }
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
        self.own(subscription);
    }

    /// Own any value and drop it when the scope is disposed.
    pub fn own<T: 'static>(&mut self, value: T) {
        self.on_cleanup(move || drop(value));
    }

    /// Run cleanups now. Idempotent; `Drop` calls it again with an empty list.
    pub fn dispose(&mut self) {
        while let Some(callback) = self.cleanups.pop() {
            callback();
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.dispose();
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
