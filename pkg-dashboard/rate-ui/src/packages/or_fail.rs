use anyhow::Error;

pub trait Fail {
    type Target;

    fn or_fail(self, reason: &'static str) -> Result<Self::Target, Error>;
}

impl<T> Fail for Option<T> {
    type Target = T;

    fn or_fail(self, reason: &'static str) -> Result<Self::Target, Error> {
        self.ok_or_else(|| Error::msg(reason))
    }
}

/*
impl<T, E> Fail for Result<T, E> {
    type Target = T;

    fn or_fail(self, reason: &'static str) -> Result<Self::Target, Error> {
        self.map_err(|_| Error::msg(reason))
    }
}
*/

pub trait Fasten {
    type Target;
    fn fasten(self) -> Result<Self::Target, Error>;
}

/*
impl<T, E: ToString> Fasten for Result<T, E> {
    type Target = T;

    fn fasten(self) -> Result<Self::Target, Error> {
        self.map_err(|err| Error::msg(err.to_string()))
    }
}
*/

impl<T> Fasten for Result<T, wasm_bindgen::JsValue> {
    type Target = T;

    fn fasten(self) -> Result<Self::Target, Error> {
        self.map_err(|err| {
            let reason = err
                .as_string()
                .unwrap_or_else(|| "unknown error inside wasm_bindgen::JsValue".into());
            Error::msg(reason)
        })
    }
}

impl<T> Fasten for Result<T, js_sys::Object> {
    type Target = T;

    fn fasten(self) -> Result<Self::Target, Error> {
        self.map_err(|err| {
            let reason = err
                .to_string()
                .as_string()
                .unwrap_or_else(|| "unknown error inside js_sys::Object".into());
            Error::msg(reason)
        })
    }
}
