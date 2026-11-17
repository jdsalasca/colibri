//! Scoped context: `provide_context` in a component body, `use_context` below it.
//!
//! Components are plain functions, so a provider is an imperative call at the
//! start of the body. `#[component]` wraps each body in [`with_context_scope`],
//! so provides never leak to siblings built later.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

type Entry = (TypeId, Rc<dyn Any>);

thread_local! {
    static CONTEXT: RefCell<Vec<Entry>> = const { RefCell::new(Vec::new()) };
}

/// Make `value` visible to components built after this call in the same body.
pub fn provide_context<T: Clone + 'static>(value: T) {
    CONTEXT.with(|stack| {
        stack.borrow_mut().push((TypeId::of::<T>(), Rc::new(value)));
    });
}

/// Read the nearest provided value of type `T`.
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    CONTEXT.with(|stack| {
        stack
            .borrow()
            .iter()
            .rev()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .and_then(|(_, value)| value.downcast_ref::<T>().cloned())
    })
}

struct ContextScopeGuard {
    len: usize,
}

impl Drop for ContextScopeGuard {
    fn drop(&mut self) {
        CONTEXT.with(|stack| stack.borrow_mut().truncate(self.len));
    }
}

/// Run `f` with a context scope that is restored when it returns or panics.
pub fn with_context_scope<R>(f: impl FnOnce() -> R) -> R {
    let len = CONTEXT.with(|stack| stack.borrow().len());
    let guard = ContextScopeGuard { len };
    let value = f();
    drop(guard);
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct Theme(&'static str);

    #[test]
    fn provides_and_reads_the_nearest_value() {
        assert_eq!(use_context::<Theme>(), None);
        with_context_scope(|| {
            provide_context(Theme("light"));
            assert_eq!(use_context::<Theme>(), Some(Theme("light")));
            with_context_scope(|| {
                provide_context(Theme("dark"));
                assert_eq!(use_context::<Theme>(), Some(Theme("dark")));
            });
            assert_eq!(use_context::<Theme>(), Some(Theme("light")));
        });
        assert_eq!(use_context::<Theme>(), None);
    }

    #[test]
    fn scope_restores_on_panic() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_context_scope(|| {
                provide_context(Theme("dark"));
                panic!("boom");
            });
        }));
        assert!(result.is_err());
        assert_eq!(use_context::<Theme>(), None);
    }

    #[test]
    fn values_are_typed() {
        with_context_scope(|| {
            provide_context(41_i32);
            provide_context(String::from("hola"));
            assert_eq!(use_context::<i32>(), Some(41));
            assert_eq!(use_context::<String>().as_deref(), Some("hola"));
            assert_eq!(use_context::<bool>(), None);
        });
    }
}
