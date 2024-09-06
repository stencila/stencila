export function eventThrottle(callback: () => void, limit: number) {
  let wait = false
  return () => {
    if (!wait) {
      callback()
      wait = true
      setTimeout(() => {
        wait = false
      }, limit)
    }
  }
}
