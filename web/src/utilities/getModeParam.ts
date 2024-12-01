/**
 * A function that takes the window instance and checks for the `?mode=` param.
 * If param exists, will return the value, otherwise returns `null`.
 */
export const getModeParam = (window: Window) => {
  const params = new URLSearchParams(window.location.search)
  const mode = params.get('mode')
  if (mode) {
    return mode
  }
  return null
}
