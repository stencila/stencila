export default function sentence_case (value) {
  value = value.replace('_', ' ')
  return value[0].toUpperCase() + value.substring(1)
}
