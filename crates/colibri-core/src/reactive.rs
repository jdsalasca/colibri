//! Fine-grained reactive computations with automatic *synchronous* dependency tracking.
//!
//! This is a small single-threaded research runtime, not a React hook renderer.
//! Computations run eagerly; effects are flushed after memos in each update turn.
//! Async reads are not tracked; there is no browser scheduler or multithreading.
//! Cycles are capped to prevent unbounded feedback loops.

use crate::{Scope, Signal};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::rc::{Rc, Weak};

const MAX_FLUSH_RUNS: usize = 4096;
type Cleanup = Box<dyn FnOnce()>;
type CleanupSlot = Rc<RefCell<Option<Cleanup>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveError {
    CycleLimit { runs: usize },
}

impl fmt::Display for ReactiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleLimit { runs } => write!(
                f,
                "reactive cycle exceeded {MAX_FLUSH_RUNS} runs ({runs} executed)"
            ),
        }
    }
}

impl Error for ReactiveError {}

/// Take the last reactive error recorded on this thread.
///
/// The runtime stops a cycle instead of panicking; callers that care about
/// diagnostics should check this after updates.
pub fn take_error() -> Option<ReactiveError> {
    LAST_ERROR.with(|slot| slot.borrow_mut().take())
}

pub(crate) fn queued_tasks() -> usize {
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        engine.memos.len() + engine.effects.len()
    })
}

#[derive(Clone, Copy)]
enum Kind {
    Memo,
    Effect,
}

struct Observer {
    kind: Kind,
    callback: RefCell<Box<dyn FnMut()>>,
    dependencies: RefCell<Vec<Box<dyn Any>>>,
    seen: RefCell<HashSet<usize>>,
    queued: Cell<bool>,
    disposed: Cell<bool>,
}

#[derive(Default)]
struct Engine {
    memos: VecDeque<Weak<Observer>>,
    effects: VecDeque<Weak<Observer>>,
    holds: usize,
    flushing: bool,
}

thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::default());
    static ACTIVE: Cell<usize> = const { Cell::new(0) };
    static TRACKING: RefCell<Vec<Weak<Observer>>> = const { RefCell::new(Vec::new()) };
    static LAST_ERROR: RefCell<Option<ReactiveError>> = const { RefCell::new(None) };
    static EFFECT_SCOPE: RefCell<Vec<Vec<Effect>>> = const { RefCell::new(Vec::new()) };
}

struct HoldGuard;

impl HoldGuard {
    fn new() -> Self {
        ENGINE.with(|engine| engine.borrow_mut().holds += 1);
        Self
    }
}

impl Drop for HoldGuard {
    fn drop(&mut self) {
        ENGINE.with(|engine| engine.borrow_mut().holds -= 1);
        flush();
    }
}

/// Defer reactive effects and memos until all manual subscribers have been notified.
pub(crate) fn notification_guard() -> impl Drop {
    HoldGuard::new()
}

/// Group several signal mutations into a single reactive propagation turn.
///
/// Explicit subscribers still receive each update; automatic effects are
/// deduplicated and run after derived memos have been processed.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    let _hold = HoldGuard::new();
    f()
}

struct UntrackGuard;

impl Drop for UntrackGuard {
    fn drop(&mut self) {
        TRACKING.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// Run a closure without registering reactive reads in its enclosing observer.
/// Useful for teardown, logging and inspecting state without dependencies.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    TRACKING.with(|stack| stack.borrow_mut().push(Weak::new()));
    let _guard = UntrackGuard;
    f()
}

struct EffectScopeGuard {
    active: bool,
}

impl Drop for EffectScopeGuard {
    fn drop(&mut self) {
        if self.active {
            EFFECT_SCOPE.with(|scopes| {
                scopes.borrow_mut().pop();
            });
        }
    }
}

/// Run `f` collecting effects created inside it. The returned effects keep
/// those effects alive until the caller drops them (for example, a mounted view).
pub fn with_effect_scope<R>(f: impl FnOnce() -> R) -> (R, Vec<Effect>) {
    EFFECT_SCOPE.with(|scopes| scopes.borrow_mut().push(Vec::new()));
    let mut guard = EffectScopeGuard { active: true };
    let result = f();
    let effects = EFFECT_SCOPE.with(|scopes| {
        scopes
            .borrow_mut()
            .pop()
            .expect("effect scope must be present")
    });
    guard.active = false;
    (result, effects)
}

fn register_effect(effect: &Effect) {
    EFFECT_SCOPE.with(|scopes| {
        if let Some(scope) = scopes.borrow_mut().last_mut() {
            scope.push(effect.clone());
        }
    });
}

struct TrackingGuard;

impl TrackingGuard {
    fn enter(observer: &Rc<Observer>) -> Self {
        ACTIVE.with(|depth| depth.set(depth.get() + 1));
        TRACKING.with(|stack| stack.borrow_mut().push(Rc::downgrade(observer)));
        Self
    }
}

impl Drop for TrackingGuard {
    fn drop(&mut self) {
        TRACKING.with(|stack| {
            stack.borrow_mut().pop();
        });
        ACTIVE.with(|depth| depth.set(depth.get() - 1));
    }
}

struct FlushGuard;

impl Drop for FlushGuard {
    fn drop(&mut self) {
        ENGINE.with(|engine| engine.borrow_mut().flushing = false);
    }
}

fn enqueue(observer: &Rc<Observer>) {
    if observer.disposed.get() || observer.queued.replace(true) {
        return;
    }
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let queue = match observer.kind {
            Kind::Memo => &mut engine.memos,
            Kind::Effect => &mut engine.effects,
        };
        queue.push_back(Rc::downgrade(observer));
    });
    flush();
}

fn flush() {
    if ACTIVE.with(|depth| depth.get() != 0) {
        return;
    }
    let started = ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        if engine.flushing || engine.holds != 0 {
            return false;
        }
        engine.flushing = true;
        true
    });
    if !started {
        return;
    }
    let _guard = FlushGuard;
    let mut runs = 0usize;
    loop {
        let next = ENGINE.with(|engine| {
            let mut engine = engine.borrow_mut();
            engine
                .memos
                .pop_front()
                .or_else(|| engine.effects.pop_front())
        });
        let Some(next) = next else { break };
        let Some(observer) = next.upgrade() else {
            continue;
        };
        observer.queued.set(false);
        if observer.disposed.get() {
            continue;
        }
        runs += 1;
        if runs > MAX_FLUSH_RUNS {
            LAST_ERROR.with(|slot| {
                *slot.borrow_mut() = Some(ReactiveError::CycleLimit { runs });
            });
            reset_queued();
            break;
        }
        execute(&observer);
    }
}

fn reset_queued() {
    ENGINE.with(|cell| {
        let (memos, effects) = {
            let mut engine = cell.borrow_mut();
            (
                std::mem::take(&mut engine.memos),
                std::mem::take(&mut engine.effects),
            )
        };
        for weak in memos.into_iter().chain(effects) {
            if let Some(observer) = weak.upgrade() {
                observer.queued.set(false);
            }
        }
    });
}

fn execute(observer: &Rc<Observer>) {
    if observer.disposed.get() {
        return;
    }
    observer.dependencies.borrow_mut().clear();
    observer.seen.borrow_mut().clear();
    let _tracking = TrackingGuard::enter(observer);
    (observer.callback.borrow_mut())();
}

/// Called by `Signal::with`; attaches the active computation to the signal.
/// Repeated reads register only one subscriber per signal per execution.
pub(crate) fn track_signal<T: 'static>(signal: &Signal<T>) {
    let current = TRACKING.with(|stack| stack.borrow().last().cloned());
    let Some(observer) = current.and_then(|weak| weak.upgrade()) else {
        return;
    };
    if observer.disposed.get() || !observer.seen.borrow_mut().insert(signal.identity()) {
        return;
    }
    let weak = Rc::downgrade(&observer);
    let subscription = signal.subscribe(move || {
        if let Some(observer) = weak.upgrade() {
            enqueue(&observer);
        }
    });
    observer
        .dependencies
        .borrow_mut()
        .push(Box::new(subscription));
}

/// Create an owned state signal; unlike React, its identity is not coupled
/// to the invocation order of component functions.
pub fn use_state<T: 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

/// Handle for a reactive effect. Dropping it unsubscribes dependencies.
///
/// An effect automatically tracks synchronous `Signal::get`/`with` calls.
/// Keep this handle in a component `Scope` or another owner.
pub struct Effect {
    observer: Rc<Observer>,
    cleanup: Option<CleanupSlot>,
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Self {
            observer: Rc::clone(&self.observer),
            cleanup: self.cleanup.clone(),
        }
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        if Rc::strong_count(&self.observer) != 1 {
            return;
        }
        self.observer.disposed.set(true);
        self.observer.dependencies.borrow_mut().clear();
        if let Some(cleanup) = &self.cleanup {
            if let Some(f) = cleanup.borrow_mut().take() {
                untrack(f);
            }
        }
        crate::observer_dropped();
    }
}

fn create_observer(kind: Kind, callback: impl FnMut() + 'static) -> Rc<Observer> {
    crate::observer_created();
    Rc::new(Observer {
        kind,
        callback: RefCell::new(Box::new(callback)),
        dependencies: RefCell::new(Vec::new()),
        seen: RefCell::new(HashSet::new()),
        queued: Cell::new(false),
        disposed: Cell::new(false),
    })
}

/// A side-effect with implicit, synchronous dependency tracking.
///
/// Executes once immediately and re-executes after relevant signal updates,
/// after pending memos are refreshed. Avoid writing to dependencies from effects.
pub fn use_effect(callback: impl FnMut() + 'static) -> Effect {
    let observer = create_observer(Kind::Effect, callback);
    execute(&observer);
    flush();
    let effect = Effect {
        observer,
        cleanup: None,
    };
    register_effect(&effect);
    effect
}

/// Effect variant with teardown before every re-run and at final disposal.
pub fn use_effect_with_cleanup(
    mut callback: impl FnMut() -> Box<dyn FnOnce()> + 'static,
) -> Effect {
    let cleanup: CleanupSlot = Rc::new(RefCell::new(None));
    let slot = Rc::clone(&cleanup);
    let observer = create_observer(Kind::Effect, move || {
        // Do not hold a RefCell borrow while user cleanup code executes.
        let previous = slot.borrow_mut().take();
        if let Some(previous) = previous {
            untrack(previous);
        }
        let next = callback();
        *slot.borrow_mut() = Some(next);
    });
    execute(&observer);
    flush();
    let effect = Effect {
        observer,
        cleanup: Some(cleanup),
    };
    register_effect(&effect);
    effect
}

struct MemoInner<T> {
    value: Signal<Option<T>>,
    _effect: Effect,
}

/// A read-only, eagerly maintained, equality-checked derived value.
///
/// The computation tracks its synchronous signal reads. Changed source
/// signals recompute the value once per batch; equal results do not notify
/// downstream observers. Compared with Leptos this is intentionally eager.
pub struct Memo<T> {
    inner: Rc<MemoInner<T>>,
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + PartialEq + 'static> Memo<T> {
    #[allow(
        clippy::expect_used,
        reason = "use_memo initializes the value before returning the handle"
    )]
    pub fn get(&self) -> T {
        self.inner
            .value
            .get()
            .expect("Colibri memo must be initialized before reading")
    }

    #[allow(
        clippy::expect_used,
        reason = "use_memo initializes the value before returning the handle"
    )]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.inner
            .value
            .with(|value| f(value.as_ref().expect("Colibri memo must be initialized")))
    }
}

/// Create a memo with automatic dependency tracking and distinct-value
/// notification. Use for expensive shared derivations, not every expression.
pub fn use_memo<T: Clone + PartialEq + 'static>(
    mut compute: impl FnMut() -> T + 'static,
) -> Memo<T> {
    let value: Signal<Option<T>> = Signal::new(None);
    let destination = value.clone();
    let observer = create_observer(Kind::Memo, move || {
        let next = compute();
        // Do not track the memo's own output while recomputing it.
        let changed = destination.peek(|old| old.as_ref() != Some(&next));
        if changed {
            destination.set(Some(next));
        }
    });
    execute(&observer);
    flush();
    Memo {
        inner: Rc::new(MemoInner {
            value,
            _effect: Effect {
                observer,
                cleanup: None,
            },
        }),
    }
}

impl Scope {
    /// Own a computation for this scope's lifetime.
    pub fn track_effect(&mut self, effect: Effect) {
        self.on_cleanup(move || drop(effect));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn use_state_and_effect_track_get_and_stop_on_drop() {
        let count = use_state(0);
        let history = Rc::new(RefCell::new(Vec::new()));
        let effect = {
            let count = count.clone();
            let history = history.clone();
            use_effect(move || history.borrow_mut().push(count.get()))
        };
        count.set(1);
        count.set(2);
        assert_eq!(*history.borrow(), vec![0, 1, 2]);
        drop(effect);
        count.set(3);
        assert_eq!(*history.borrow(), vec![0, 1, 2]);
        assert_eq!(count.subscriber_count(), 0);
    }

    #[test]
    fn memo_caches_until_source_changes_and_filters_equal_outputs() {
        let count = use_state(1);
        let calls = Rc::new(Cell::new(0));
        let memo = {
            let count = count.clone();
            let calls = calls.clone();
            use_memo(move || {
                calls.set(calls.get() + 1);
                count.get() % 2
            })
        };
        assert_eq!(memo.get(), 1);
        assert_eq!(memo.get(), 1);
        assert_eq!(calls.get(), 1);
        let observed = Rc::new(Cell::new(0));
        let effect = {
            let memo = memo.clone();
            let observed = observed.clone();
            use_effect(move || {
                memo.get();
                observed.set(observed.get() + 1);
            })
        };
        count.set(3);
        assert_eq!(observed.get(), 1);
        count.set(4);
        assert_eq!(observed.get(), 2);
        assert_eq!(memo.get(), 0);
        drop(effect);
        drop(memo);
    }

    #[test]
    fn dynamic_dependencies_are_replaced() {
        let select_left = use_state(true);
        let left = use_state(10);
        let right = use_state(20);
        let observed = Rc::new(RefCell::new(Vec::new()));
        let effect = {
            let select_left = select_left.clone();
            let left = left.clone();
            let right = right.clone();
            let observed = observed.clone();
            use_effect(move || {
                let value = if select_left.get() {
                    left.get()
                } else {
                    right.get()
                };
                observed.borrow_mut().push(value);
            })
        };
        select_left.set(false);
        left.set(11);
        right.set(21);
        assert_eq!(*observed.borrow(), vec![10, 20, 21]);
        assert_eq!(left.subscriber_count(), 0);
        drop(effect);
        assert_eq!(right.subscriber_count(), 0);
    }

    #[test]
    fn batching_and_memo_priority_avoid_intermediate_glitches() {
        let left = use_state(1);
        let right = use_state(2);
        let total = {
            let left = left.clone();
            let right = right.clone();
            use_memo(move || left.get() + right.get())
        };
        let values = Rc::new(RefCell::new(Vec::new()));
        let effect = {
            let left = left.clone();
            let total = total.clone();
            let values = values.clone();
            use_effect(move || {
                values.borrow_mut().push((left.get(), total.get()));
            })
        };
        batch(|| {
            left.set(4);
            right.set(5);
        });
        assert_eq!(*values.borrow(), vec![(1, 3), (4, 9)]);
        drop(effect);
    }

    #[test]
    fn cleanup_runs_before_rerun_and_when_disposed() {
        let signal = use_state(0);
        let events = Rc::new(RefCell::new(Vec::new()));
        let effect = {
            let signal = signal.clone();
            let events = events.clone();
            use_effect_with_cleanup(move || {
                let v = signal.get();
                events.borrow_mut().push(format!("run:{v}"));
                let events = events.clone();
                Box::new(move || events.borrow_mut().push(format!("cleanup:{v}")))
            })
        };
        signal.set(1);
        drop(effect);
        assert_eq!(
            *events.borrow(),
            vec!["run:0", "cleanup:0", "run:1", "cleanup:1"]
        );
    }

    #[test]
    fn effect_cleanup_reads_are_not_tracked_as_dependencies() {
        let source = use_state(0);
        let unrelated = use_state(0);
        let runs = Rc::new(Cell::new(0));
        let effect = {
            let source = source.clone();
            let unrelated = unrelated.clone();
            let runs = runs.clone();
            use_effect_with_cleanup(move || {
                source.get();
                runs.set(runs.get() + 1);
                let unrelated = unrelated.clone();
                Box::new(move || {
                    unrelated.get();
                })
            })
        };
        source.set(1);
        assert_eq!(runs.get(), 2);
        unrelated.set(1);
        assert_eq!(runs.get(), 2);
        drop(effect);
        assert_eq!(unrelated.subscriber_count(), 0);
    }

    #[test]
    fn scope_tears_down_many_effects() {
        let signal = use_state(0);
        let calls = Rc::new(Cell::new(0));
        for _ in 0..10_000 {
            let mut scope = Scope::new();
            let signal = signal.clone();
            let calls = calls.clone();
            scope.track_effect(use_effect(move || {
                signal.get();
                calls.set(calls.get() + 1);
            }));
        }
        assert_eq!(signal.subscriber_count(), 0);
        let snapshot = calls.get();
        signal.set(1);
        assert_eq!(calls.get(), snapshot);
    }

    #[test]
    fn update_from_effect_is_queued_without_recursive_borrow() {
        let signal = use_state(0);
        let effect = {
            let signal = signal.clone();
            use_effect(move || {
                let value = signal.get();
                if value < 3 {
                    signal.set(value + 1);
                }
            })
        };
        assert_eq!(signal.get(), 3);
        drop(effect);
    }

    #[test]
    fn reactive_cycles_record_an_error_and_the_queue_recovers() {
        let _ = take_error();
        let left = use_state(0);
        let right = use_state(0);
        let first = {
            let left = left.clone();
            let right = right.clone();
            use_effect(move || {
                let value = left.get();
                if value > 0 {
                    right.set(value);
                }
            })
        };
        let second = {
            let right = right.clone();
            let left = left.clone();
            use_effect(move || {
                let value = right.get();
                if value > 0 {
                    left.set(value);
                }
            })
        };

        left.set(1);
        assert!(matches!(
            take_error(),
            Some(ReactiveError::CycleLimit { .. })
        ));
        drop(first);
        drop(second);

        let signal = use_state(0);
        let runs = Rc::new(Cell::new(0));
        let effect = {
            let signal = signal.clone();
            let runs = runs.clone();
            use_effect(move || {
                signal.get();
                runs.set(runs.get() + 1);
            })
        };
        signal.set(1);
        assert_eq!(runs.get(), 2);
        assert!(take_error().is_none());
        drop(effect);
    }

    #[test]
    fn effect_clone_survives_until_last_handle() {
        let signal = use_state(0);
        let runs = Rc::new(Cell::new(0));
        let effect = {
            let signal = signal.clone();
            let runs = runs.clone();
            use_effect(move || {
                signal.get();
                runs.set(runs.get() + 1);
            })
        };
        let clone = effect.clone();
        drop(effect);
        signal.set(1);
        assert_eq!(runs.get(), 2);
        drop(clone);
        signal.set(2);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn effect_scope_owns_effects_created_inside() {
        let signal = use_state(0);
        let runs = Rc::new(Cell::new(0));
        let (value, effects) = with_effect_scope(|| {
            let signal = signal.clone();
            let runs = runs.clone();
            use_effect(move || {
                signal.get();
                runs.set(runs.get() + 1);
            });
            42
        });
        assert_eq!(value, 42);
        assert_eq!(runs.get(), 1);
        signal.set(1);
        assert_eq!(runs.get(), 2);
        drop(effects);
        signal.set(2);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn ten_thousand_mixed_cycles_leave_no_residuals() {
        let baseline = crate::stats();
        for _ in 0..10_000 {
            let signal = use_state(0);
            let memo = {
                let signal = signal.clone();
                use_memo(move || signal.get() * 2)
            };
            let effect = {
                let memo = memo.clone();
                use_effect(move || {
                    memo.get();
                })
            };
            signal.set(1);
            assert_eq!(memo.get(), 2);
            drop(effect);
            drop(memo);
            drop(signal);
        }
        assert_eq!(crate::stats(), baseline);
    }

    #[test]
    fn scope_dispose_is_lifo_and_idempotent() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut scope = Scope::new();
        let first = events.clone();
        scope.on_cleanup(move || first.borrow_mut().push("first"));
        let second = events.clone();
        scope.on_cleanup(move || second.borrow_mut().push("second"));

        scope.dispose();
        assert_eq!(*events.borrow(), vec!["second", "first"]);
        scope.dispose();
        assert_eq!(*events.borrow(), vec!["second", "first"]);
    }
}
