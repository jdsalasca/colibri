use colibri_core::{
    effect_scope_active, settle_resource, use_effect_with_cleanup, Resource, Signal,
};
use std::cell::Cell;
use std::future::Future;
use std::rc::Rc;

/// Spawn `future` on the browser event loop and expose its state as a signal.
///
/// The signal starts in [`Resource::Loading`] and becomes `Ready` or `Failed`
/// when the future completes. Inside a component (effect scope), unmounting
/// cancels the update: the result is dropped instead of writing the signal.
/// Outside a scope the future runs to completion; there is no mount to cancel.
pub fn use_async<T, F>(future: F) -> Signal<Resource<T>>
where
    T: Clone + 'static,
    F: Future<Output = Result<T, String>> + 'static,
{
    let state = Signal::new(Resource::Loading);
    let target = state.clone();
    let cancelled = Rc::new(Cell::new(false));
    if effect_scope_active() {
        let flag = Rc::clone(&cancelled);
        use_effect_with_cleanup(move || {
            let flag = Rc::clone(&flag);
            Box::new(move || flag.set(true))
        });
    }
    wasm_bindgen_futures::spawn_local(async move {
        let result = future.await;
        settle_resource(&target, &cancelled, result);
    });
    state
}
