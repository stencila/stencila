/**
 * A function that takes the window instance and checks for the `?mode=` param
 * Will retunr the value is it is there, otherwise `null`
 */
export const getModeParam = (window: Window) => {
  const params = new URLSearchParams(window.location.search)
  const mode = params.get('mode')
  if (mode) {
    return mode
  }
  return null
}
