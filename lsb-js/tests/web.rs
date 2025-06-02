//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use lsb_js::ExtractResult;
use wasm_bindgen_test::*;

// wasm_bindgen_test_configure!(run_in_browser);

const INPUT: &[u8] = include_bytes!("../../data/input.webp");
const EXTENSION: &str = "webp";
const CONTAINER: &[u8] = include_bytes!("../../data/container.webp");
const EMBEDDED: &[u8] = include_bytes!("../../data/embedded.png");

#[wasm_bindgen_test]
fn test_embed() {
    let result = lsb_js::embed(
        INPUT,
        EXTENSION,
        CONTAINER,
        Some(1),
        Some("BLAKE3".to_string()),
        Some(42),
        Some("PNG".to_string()),
    );
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_extract() {
    let result = lsb_js::extract(EMBEDDED, Some(1), Some(42));
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_embed_extract() -> Result<(), Box<dyn std::error::Error>> {
    let result = lsb_js::embed(
        INPUT,
        EXTENSION,
        CONTAINER,
        Some(1),
        Some("BLAKE3".to_string()),
        Some(42),
        Some("PNG".to_string()),
    );
    assert!(result.is_ok());
    let result = result?;

    let result = lsb_js::extract(&result, Some(1), Some(42));
    assert!(result.is_ok());
    let ExtractResult(result, _) = result?;
    assert_eq!(result, INPUT);

    Ok(())
}
