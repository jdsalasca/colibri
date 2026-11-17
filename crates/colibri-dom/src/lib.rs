//! Minimal browser runtime for Colibrí views.
//!
//! The public API is available on `wasm32` only. `mount` builds DOM nodes once,
//! updates dynamic text/attributes through `colibri-core` effects, and removes
//! listeners and effects when the mount handle is dropped.

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(target_arch = "wasm32")]
mod resource;

#[cfg(target_arch = "wasm32")]
pub use browser::{
    attach_effects, mount, Attribute, ElementView, EventBinding, KeyedView, MountHandle, View,
};
#[cfg(target_arch = "wasm32")]
pub use resource::use_async;
