#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::spawn(f);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(f: F)
where
    F: FnOnce() + 'static,
{
    wasm_bindgen_futures::spawn_local(async move {
        f();
    });
}
