export default function random_generate_(n, generator) {
  if (n === 1) return generator()
  else {
    let nums = []
    for (let i = 0; i < n; i++) {
      nums.push(generator())
    }
    return nums
  }
}
