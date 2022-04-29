use wasm_bindgen::JsValue;
use gloo::console::error;
use js_sys::Function;
pub use js_sys::Promise;

pub struct PendingPromise {
    resolve: Function,
    reject: Function,
}

impl PendingPromise {
    pub fn new() -> (Promise, PendingPromise) {
        let mut resolve = None;
        let mut reject = None;
        let promise = Promise::new(&mut |res, rej| {
            resolve = Some(res);
            reject = Some(rej);
        });
        let resolve = resolve.unwrap();
        let reject = reject.unwrap();
        (promise, PendingPromise {resolve, reject})
    }

    pub fn resolve<OK>(self, ok: OK)
    where
        JsValue: From<OK>
    {
        if let Err(js_err) = self.resolve.call1(&JsValue::NULL, &JsValue::from(ok)) {
            log_err("Couldn't resolve promise:", js_err);
        }
    }

    pub fn reject<ERR>(self, err: ERR)
    where
        JsValue: From<ERR>
    {
        if let Err(js_err) = self.reject.call1(&JsValue::NULL, &JsValue::from(err)) {
            log_err("Couldn't reject promise", js_err);
        }
    }

    pub fn finish<OK, ERR>(self, result: Result<OK, ERR>)
    where
        JsValue: From<OK>,
        JsValue: From<ERR>,
    {
        match result {
            Ok(ok) => self.resolve(ok),
            Err(err) => self.reject(err),
        };
    }
}

// Got weird type errors when inlining the macro
fn log_err(msg: &'static str, js_err: JsValue) {
    error!(msg, js_err);
}
