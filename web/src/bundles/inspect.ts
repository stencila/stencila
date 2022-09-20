/**
 * Browser bundle for user mode `Inspect`
 *
 * Adds web components necessary for inspecting (and modifying) the
 * execution of a document (e.g. viewing code of code chunks)
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./interact').then(() => elevateMode(Mode.Inspect)).catch(console.error)

import '../components/nodes/code-chunk'
// import '../components/nodes/code-expression'
// import '../components/nodes/include'
// import '../components/nodes/call'
