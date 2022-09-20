/**
 * The current user mode
 */
export const enum Mode {
  Read = 0,
  View = 1,
  Interact = 2,
  Inspect = 3,
  Modify = 4,
  Edit = 5,
  Write = 6,
}

/**
 * Set the current user mode
 */
export function setMode(mode: Mode) {
  window.stencila = { ...window.stencila, mode }
}

/**
 * Elevate the current user mode
 *
 * Will only set the mode if the existing mode is undefined or
 * lower than the current mode. This function was necessary for
 * guaranteeing the final mode is as expected when doing a chain of
 * dynamic, async `import`s for browser bundles.
 */
export function elevateMode(mode: Mode) {
  if ((window.stencila?.mode ?? Mode.Read) < mode) {
    window.stencila = { ...window.stencila, mode }
  }
}

/**
 * Get the current user mode
 */
export function getMode(): Mode {
  return window.stencila.mode ?? Mode.Read
}
