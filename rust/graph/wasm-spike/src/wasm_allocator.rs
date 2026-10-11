//! Replace the upstream scanner-only bump allocator with Rust's allocator.
//!
//! All five libc allocation/abort symbols are supplied together so the linker
//! never pulls tree-sitter-language's stdlib.c archive member into the module.
//! The prefix records the layout needed by Rust to free or resize C allocations.

use std::{
    alloc::{Layout, alloc, dealloc, realloc as rust_realloc},
    ffi::c_void,
    ptr,
};

const ALIGN: usize = 16;
const HEADER: usize = ALIGN;

fn layout(size: usize) -> Option<Layout> {
    Layout::from_size_align(size.max(1).checked_add(HEADER)?, ALIGN).ok()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let Some(layout) = layout(size) else {
        return ptr::null_mut();
    };
    // SAFETY: Layout is valid; null is checked before writing the prefix.
    let base = unsafe { alloc(layout) };
    if base.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        base.cast::<usize>().write(layout.size());
        base.add(HEADER).cast()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut c_void {
    let Some(size) = count.checked_mul(size) else {
        return ptr::null_mut();
    };
    let pointer = unsafe { malloc(size) };
    if !pointer.is_null() {
        // SAFETY: malloc allocated at least size writable bytes.
        unsafe { pointer.cast::<u8>().write_bytes(0, size) };
    }
    pointer
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(pointer: *mut c_void) {
    if pointer.is_null() {
        return;
    }
    // SAFETY: C only frees pointers allocated by these functions; their prefix
    // stores the valid size and uses the same fixed alignment.
    unsafe {
        let base = pointer.cast::<u8>().sub(HEADER);
        let size = base.cast::<usize>().read();
        let layout = Layout::from_size_align_unchecked(size, ALIGN);
        dealloc(base, layout);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(pointer: *mut c_void, size: usize) -> *mut c_void {
    if pointer.is_null() {
        return unsafe { malloc(size) };
    }
    if size == 0 {
        unsafe { free(pointer) };
        return ptr::null_mut();
    }
    let Some(new_layout) = layout(size) else {
        return ptr::null_mut();
    };
    // SAFETY: The original prefix describes the existing allocation. Rust's
    // realloc preserves that allocation on failure; success updates the prefix.
    unsafe {
        let base = pointer.cast::<u8>().sub(HEADER);
        let old_size = base.cast::<usize>().read();
        let old_layout = Layout::from_size_align_unchecked(old_size, ALIGN);
        let new_base = rust_realloc(base, old_layout, new_layout.size());
        if new_base.is_null() {
            return ptr::null_mut();
        }
        new_base.cast::<usize>().write(new_layout.size());
        new_base.add(HEADER).cast()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    core::arch::wasm32::unreachable()
}
