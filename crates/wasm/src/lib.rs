use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn ping() -> String {
    rus_core::placeholder();
    "pong".to_string()
}
