/**
 * Browser bundle for user mode `Interact`
 *
 * Adds web components necessary for interacting with the document
 * but not for inspecting of modifying its execution.
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./view').then(() => elevateMode(Mode.Interact)).catch(console.error)

// import '../components/nodes/parameter'
// import '../components/nodes/filter'
// import '../components/nodes/gate'
// import '../components/nodes/form'
