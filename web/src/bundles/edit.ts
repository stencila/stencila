/**
 * Browser bundle for user mode `Edit`
 *
 * Add editor for document content.
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./modify').then(() => elevateMode(Mode.Edit)).catch(console.error)
