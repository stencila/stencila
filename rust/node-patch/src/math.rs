//! Patching for math related nodes

use stencila_schema::*;

use super::prelude::*;

patchable_struct!(
    MathBlock,
    id,
    math_language,
    code,
    compile_digest,
    errors,
    mathml
);

patchable_struct!(
    MathFragment,
    id,
    math_language,
    code,
    compile_digest,
    errors,
    mathml
);
