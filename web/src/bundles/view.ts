/**
 * Browser bundle for user mode `View`
 *
 * Adds the client so that patches can be received (but not sent) so that
 * the page represents a "live view" of the document.
 */

import { Mode, elevateMode } from '../mode'

// @ts-ignore
import('./read').then(() => elevateMode(Mode.View)).catch(console.error)

import '../client'
