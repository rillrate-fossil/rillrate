use web_sys::HtmlElement;
use yew::NodeRef;

/*
use anyhow::Error;
use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Debug, Error)]
enum Reason {
    #[error("can't cast the node to HtmlElement")]
    NotHtmlElement,
}

pub fn js_err(value: JsValue) -> Error {
    Error::msg(value.as_string().unwrap_or_else(|| "js error".into()))
}
*/

/// Updates node directly by `NodeRef`
pub fn set_node(node_ref: &mut NodeRef, value: impl ToString) {
    let value = value.to_string();
    if let Some(node) = node_ref.cast::<HtmlElement>() {
        node.set_inner_text(&value);
    } else {
        log::error!(
            "Can't cast node {:?} to HtmlElement set it to {}",
            node_ref,
            value
        );
    }
}

pub fn set_style(node_ref: &mut NodeRef, key: &str, value: &str) {
    if let Some(node) = node_ref.cast::<HtmlElement>() {
        if let Err(_err) = node.style().set_property(key, value) {
            log::error!("Can't set property of {:?} to {}={}", node_ref, key, value);
        }
    } else {
        log::error!(
            "Can't cast node {:?} to HtmlElement to update style {}={}",
            node_ref,
            key,
            value
        );
    }
}

/*
pub fn remove_class(node_ref: &mut NodeRef, class: &str) {
    use js_sys::Array;
    use wasm_bindgen::JsValue;

    if let Some(node) = node_ref.cast::<HtmlElement>() {
        let array = Array::new();
        array.push(&JsValue::from_str(class));
        node.class_list().remove(&array);
    }
}
*/
