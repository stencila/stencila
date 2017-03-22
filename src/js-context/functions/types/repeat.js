export default function repeat (value, times) {
  let values = []
  for (let i = 0; i < times; i++) {
    values.push(value)
  }
  return values
}
