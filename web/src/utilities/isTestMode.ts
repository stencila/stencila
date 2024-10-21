/**
 * A function that takes the window instance and checks for the `?mode=test` param
 * will return true if param exists and === 'test'. Returns `false` in all other instances.
 */
export const isTestMode = (window: Window) => {
  const params = new URLSearchParams(window.location.search)
  const mode = params.get('mode')
  if (mode && mode === 'test') {
    return true
  }
  return false
}
