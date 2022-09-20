/**
 * Browser bundle for user mode `Inspect`
 *
 * Adds web components necessary for inspecting (and modifying) the
 * execution of a document (e.g. viewing code of code chunks)
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./interact').then(() => elevateMode(Mode.Inspect)).catch(console.error)

export { default as StencilaCodeChunk } from '../components/nodes/code-chunk'
//export { default as StencilaCodeExpression } from './nodes/code-expression'
//export { default as StencilaInclude } from './nodes/include'
//export { default as StencilaCall } from './nodes/call'
