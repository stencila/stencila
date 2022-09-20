/**
 * Browser bundle for user mode `Write`
 *
 * Sets the mode to `Write` to allow user to modify code and content.
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./edit').then(() => elevateMode(Mode.Write)).catch(console.error)
