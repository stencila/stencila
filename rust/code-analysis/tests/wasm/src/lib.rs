use serde::Deserialize;
use stencila_code_analysis::{ANALYZER_IDENTITY, CodeLanguage, analyze_file};

#[derive(Deserialize)]
pub struct Source {
    path: String,
    language: CodeLanguage,
    source: String,
}

pub fn analyze_json(input: &str) -> Result<String, serde_json::Error> {
    let source: Source = serde_json::from_str(input)?;
    let analysis = analyze_file(source.language, &source.path, &source.source);
    serde_json::to_string(&serde_json::json!({"analysis": analysis, "identity": ANALYZER_IDENTITY}))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn analyze(input: &str) -> Result<String, wasm_bindgen::JsValue> {
    analyze_json(input).map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn detects(path: &str) -> Option<String> {
    CodeLanguage::from_path(path).map(|language| language.id_component().to_string())
}

// Exercise the actual linked C ABI, including edge cases a source fixture does
// not reliably reach. This test-only module does not define any allocator.
#[cfg(target_arch = "wasm32")]
mod allocator_test {
    use std::{ffi::c_void, ptr};
    use wasm_bindgen::prelude::*;

    unsafe extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn calloc(count: usize, size: usize) -> *mut c_void;
        fn realloc(pointer: *mut c_void, size: usize) -> *mut c_void;
        fn free(pointer: *mut c_void);
    }

    #[wasm_bindgen]
    pub fn allocator_smoke() -> Result<(), JsValue> {
        // LLVM recognizes libc allocation calls. Opaque function pointers keep
        // it from replacing these edge-case probes with builtin assumptions.
        let malloc = std::hint::black_box(malloc as unsafe extern "C" fn(usize) -> *mut c_void);
        let calloc =
            std::hint::black_box(calloc as unsafe extern "C" fn(usize, usize) -> *mut c_void);
        let realloc = std::hint::black_box(
            realloc as unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void,
        );
        let free = std::hint::black_box(free as unsafe extern "C" fn(*mut c_void));
        // SAFETY: Only live allocations from the linked C allocator are used;
        // accesses are within their requested size. Failed realloc preserves
        // the old pointer. All live pointers are freed exactly once.
        unsafe {
            free(ptr::null_mut());
            if !calloc(usize::MAX, 2).is_null() || !malloc(usize::MAX).is_null() {
                return Err(JsValue::from_str("overflow allocations must return null"));
            }
            let zero = calloc(0, 8);
            if zero.is_null() {
                return Err(JsValue::from_str("zero-size allocation failed"));
            }
            free(zero);
            let pointer = calloc(64, 1).cast::<u8>();
            if pointer.is_null() {
                return Err(JsValue::from_str("calloc failed"));
            }
            if pointer as usize % 16 != 0 || (0..64).any(|index| pointer.add(index).read() != 0) {
                free(pointer.cast());
                return Err(JsValue::from_str(
                    "calloc alignment or zero initialization failed",
                ));
            }
            pointer.write(37);
            if !realloc(pointer.cast(), usize::MAX).is_null() {
                free(pointer.cast());
                return Err(JsValue::from_str(
                    "overflow realloc must preserve the old allocation",
                ));
            }
            let grown = realloc(pointer.cast(), 8 * 1024 * 1024).cast::<u8>();
            if grown.is_null() {
                free(pointer.cast());
                return Err(JsValue::from_str("8 MiB growth failed"));
            }
            if grown.read() != 37 {
                free(grown.cast());
                return Err(JsValue::from_str("growth lost existing bytes"));
            }
            grown.add(8 * 1024 * 1024 - 1).write(91);
            let shrunk = realloc(grown.cast(), 1).cast::<u8>();
            if shrunk.is_null() {
                free(grown.cast());
                return Err(JsValue::from_str("shrink failed"));
            }
            let retained = shrunk.read() == 37;
            if !realloc(shrunk.cast(), 0).is_null() {
                return Err(JsValue::from_str("zero-size realloc must return null"));
            }
            let from_null = realloc(ptr::null_mut(), 32);
            if from_null.is_null() {
                return Err(JsValue::from_str("realloc from null failed"));
            }
            free(from_null);
            if retained {
                Ok(())
            } else {
                Err(JsValue::from_str("shrink lost existing bytes"))
            }
        }
    }
}
