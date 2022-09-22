/**
 * Browser bundle for user mode `Inspect`
 *
 * Adds web components necessary for inspecting (and modifying) the
 * execution of a document (e.g. viewing code of code chunks)
 */

// @ts-ignore
import('./interact-imports')

export { default as StencilaCodeChunk } from '../components/nodes/code-chunk'
//export { default as StencilaCodeExpression } from './nodes/code-expression'
//export { default as StencilaInclude } from './nodes/include'
//export { default as StencilaCall } from './nodes/call'
