//! Minimal single-threaded reactive primitives for experimental research.
//! Automatic synchronous dependencies and batching live in `reactive`.
//! Browser rendering, async tracking and hydration are still unimplemented.
mod reactive;
pub use reactive::{
    batch, effect_scope_active, take_error, untrack, use_effect, use_effect_with_cleanup, use_memo,
    use_state, with_effect_scope, Effect, Memo, ReactiveError,
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

/// Async data state for `use_async` in the browser runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource<T> {
    /// The request is still running.
    Loading,
    /// The request completed with a value.
    Ready(T),
    /// The request failed with a message.
    Failed(String),
}

/// Write an async outcome into `target` unless its owner was cancelled.
///
/// Cancellation is cooperative: the future still runs, but a stale result never
/// reaches the signal (and therefore never touches a detached DOM tree).
pub fn settle_resource<T: Clone + 'static>(
    target: &Signal<Resource<T>>,
    cancelled: &Cell<bool>,
    result: Result<T, String>,
) {
    if cancelled.get() {
        return;
    }
    target.set(match result {
        Ok(value) => Resource::Ready(value),
        Err(message) => Resource::Failed(message),
    });
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

struct LazyInner<T> {
    value: RefCell<Option<T>>,
    compute: Box<dyn Fn() -> T>,
}

/// A value computed on first read and cached for the lifetime of the handle.
pub struct Lazy<T> {
    inner: Rc<LazyInner<T>>,
}

impl<T> Clone for Lazy<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone> Lazy<T> {
    /// Return the cached value, computing it on first access.
    pub fn get(&self) -> T {
        if let Some(value) = self.inner.value.borrow().as_ref() {
            return value.clone();
        }
        let value = (self.inner.compute)();
        *self.inner.value.borrow_mut() = Some(value.clone());
        value
    }
}

/// Create a lazy value. The closure runs at most once per handle.
pub fn use_lazy<T: Clone + 'static>(compute: impl Fn() -> T + 'static) -> Lazy<T> {
    Lazy {
        inner: Rc::new(LazyInner {
            value: RefCell::new(None),
            compute: Box::new(compute),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settle_resource_ignores_cancelled_results() {
        let state = Signal::new(Resource::<i32>::Loading);
        let cancelled = Cell::new(false);
        settle_resource(&state, &cancelled, Ok(7));
        assert_eq!(state.get(), Resource::Ready(7));
        cancelled.set(true);
        settle_resource(&state, &cancelled, Ok(9));
        assert_eq!(state.get(), Resource::Ready(7));
        settle_resource(&state, &cancelled, Err("late".to_string()));
        assert_eq!(state.get(), Resource::Ready(7));
    }

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

    #[test]
    fn lazy_computes_once_and_shares() {
        let calls = Rc::new(Cell::new(0));
        let lazy = use_lazy({
            let calls = Rc::clone(&calls);
            move || {
                calls.set(calls.get() + 1);
                42
            }
        });
        let clone = lazy.clone();
        assert_eq!(calls.get(), 0);
        assert_eq!(lazy.get(), 42);
        assert_eq!(clone.get(), 42);
        assert_eq!(lazy.get(), 42);
        assert_eq!(calls.get(), 1);
    }
}
