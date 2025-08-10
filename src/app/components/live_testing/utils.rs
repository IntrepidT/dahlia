use leptos::prelude::*;
use leptos::prelude::*;
use std::time::Duration as StdDuration;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue; // Add this import

#[cfg(feature = "hydrate")]
pub fn set_interval_with_handle<F>(f: F, delay: StdDuration) -> Result<IntervalHandle, JsValue>
where
    F: Fn() + 'static,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let callback = Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    let handle = web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay.as_millis() as i32,
        )?;

    callback.forget();
    Ok(IntervalHandle(handle))
}

#[cfg(feature = "hydrate")]
pub fn set_timeout_with_handle<F>(f: F, delay: StdDuration) -> Result<TimeoutHandle, JsValue>
where
    F: FnOnce() + 'static,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let callback = Closure::once(Box::new(f) as Box<dyn FnOnce()>);
    let handle = web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay.as_millis() as i32,
        )?;

    callback.forget();
    Ok(TimeoutHandle(handle))
}

#[cfg(feature = "hydrate")]
pub struct IntervalHandle(i32);

#[cfg(feature = "hydrate")]
impl IntervalHandle {
    pub fn clear(&self) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(self.0);
    }
}

#[cfg(feature = "hydrate")]
pub struct TimeoutHandle(i32);

#[cfg(feature = "hydrate")]
impl TimeoutHandle {
    pub fn clear(&self) {
        web_sys::window().unwrap().clear_timeout_with_handle(self.0);
    }
}

// Non-hydrate versions (no-ops)
#[cfg(not(feature = "hydrate"))]
pub fn set_interval_with_handle<F>(_f: F, _delay: StdDuration) -> Result<IntervalHandle, ()>
where
    F: Fn() + 'static,
{
    Ok(IntervalHandle)
}

#[cfg(not(feature = "hydrate"))]
pub fn set_timeout_with_handle<F>(_f: F, _delay: StdDuration) -> Result<TimeoutHandle, ()>
where
    F: FnOnce() + 'static,
{
    Ok(TimeoutHandle)
}

#[cfg(not(feature = "hydrate"))]
pub struct IntervalHandle;

#[cfg(not(feature = "hydrate"))]
impl IntervalHandle {
    pub fn clear(&self) {}
}

#[cfg(not(feature = "hydrate"))]
pub struct TimeoutHandle;

#[cfg(not(feature = "hydrate"))]
impl TimeoutHandle {
    pub fn clear(&self) {}
}
