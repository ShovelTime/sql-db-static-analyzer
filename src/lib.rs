mod utils;
mod heuristics;
mod binding;
use wasm_bindgen::prelude::*;

//handle errors or get annoyed at by compiler warnings 
#[warn(clippy::unwrap_used)]
#[warn(clippy::unwrap_in_result)]

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, sql-static-analyzer!");
}
