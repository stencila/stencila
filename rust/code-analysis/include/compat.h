#ifndef STENCILA_CODE_ANALYSIS_COMPAT_H
#define STENCILA_CODE_ANALYSIS_COMPAT_H

#if !defined(__wasm32__)
#error "This compatibility header is for wasm32 only"
#endif

/* Supplements tree-sitter-language 0.1.7's bare wasm headers.
 * R includes wctype.h before stdbool.h; MATLAB's scanner calls strcmp,
 * which the bundled string shim does not provide. No host imports needed. */
#include <stdbool.h>

static inline int strcmp(const char *left, const char *right) {
    while (*left && (unsigned char)*left == (unsigned char)*right) {
        left++;
        right++;
    }
    return (unsigned char)*left - (unsigned char)*right;
}

/* The upstream assert header defines an external function in every translation
 * unit. Preserve trapping assertions without duplicate __assert_fail symbols. */
#define TREE_SITTER_WASM_ASSERT_H_
#define assert(expression) ((expression) ? (void)0 : __builtin_trap())

#endif
