use colibri_core::{Resource, Signal};
use std::future::Future;

/// Spawn `future` on the browser event loop and expose its state as a signal.
///
/// The signal starts in [`Resource::Loading`] and becomes `Ready` or `Failed`
/// when the future completes. Effects that read it update the DOM.
pub fn use_async<T, F>(future: F) -> Signal<Resource<T>>
where
    T: Clone + 'static,
    F: Future<Output = Result<T, String>> + 'static,
{
    let state = Signal::new(Resource::Loading);
    let target = state.clone();
    wasm_bindgen_futures::spawn_local(async move {
        target.set(match future.await {
            Ok(value) => Resource::Ready(value),
            Err(message) => Resource::Failed(message),
        });
    });
    state
}
