//! Hydration state: read the SSR seed script and initialize hooks from it.
//!
//! The server writes `<script type="application/json" id="colibri-state">` with
//! a JSON object (see `colibri_ssr::state_script`). Components then use
//! [`use_hydrated_state`] instead of `use_state` to keep the server value.

use colibri_core::{use_state, Signal};
use std::cell::RefCell;
use wasm_bindgen::JsValue;

pub const STATE_ID: &str = "colibri-state";

thread_local! {
    static SEEDS: RefCell<Option<Option<JsValue>>> = const { RefCell::new(None) };
}

fn seeds() -> Option<JsValue> {
    SEEDS.with(|cached| {
        let mut cached = cached.borrow_mut();
        if let Some(seeds) = cached.as_ref() {
            return seeds.clone();
        }
        let parsed = read_script();
        *cached = Some(parsed.clone());
        parsed
    })
}

fn read_script() -> Option<JsValue> {
    let document = web_sys::window()?.document()?;
    let script = document.get_element_by_id(STATE_ID)?;
    let json = script.text_content()?;
    match js_sys::JSON::parse(&json) {
        Ok(value) if value.is_object() => Some(value),
        Ok(_) => {
            web_sys::console::error_1(&JsValue::from_str(
                "colibri-state must contain a JSON object",
            ));
            None
        }
        Err(error) => {
            web_sys::console::error_1(&error);
            None
        }
    }
}

fn lookup(key: &str) -> Option<JsValue> {
    let seeds = seeds()?;
    let value = js_sys::Reflect::get(&seeds, &JsValue::from_str(key)).ok()?;
    if value.is_undefined() {
        None
    } else {
        Some(value)
    }
}

/// A type that can be read from the SSR state script.
pub trait Seed: Sized {
    /// Convert a JSON value, or `None` when the type does not match.
    fn from_seed(value: &JsValue) -> Option<Self>;
}

impl Seed for i32 {
    fn from_seed(value: &JsValue) -> Option<Self> {
        let number = value.as_f64()?;
        if number.fract() != 0.0 || number < i32::MIN as f64 || number > i32::MAX as f64 {
            return None;
        }
        Some(number as i32)
    }
}

impl Seed for f64 {
    fn from_seed(value: &JsValue) -> Option<Self> {
        value.as_f64()
    }
}

impl Seed for bool {
    fn from_seed(value: &JsValue) -> Option<Self> {
        value.as_bool()
    }
}

impl Seed for String {
    fn from_seed(value: &JsValue) -> Option<Self> {
        value.as_string()
    }
}

/// State initialized from the SSR seed script, falling back to `initial`.
///
/// Reads `<script type="application/json" id="colibri-state">` once. A missing
/// key or a type mismatch uses `initial` (the mismatch is reported to console).
pub fn use_hydrated_state<T: Seed + 'static>(key: &str, initial: T) -> Signal<T> {
    if let Some(value) = lookup(key) {
        if let Some(seed) = T::from_seed(&value) {
            return use_state(seed);
        }
        web_sys::console::error_1(&JsValue::from_str(&format!(
            "colibri-state value for `{key}` has the wrong type; using initial"
        )));
    }
    use_state(initial)
}
