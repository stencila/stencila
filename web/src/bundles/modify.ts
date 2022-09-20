/**
 * Browser bundle for user mode `Modify`
 *
 * Set the mode to `Modify` so that web components allow editing
 * of code, changing of document calls etc.
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./inspect').then(() => elevateMode(Mode.Modify)).catch(console.error)
