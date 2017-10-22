export function getCellState(cell) {
  return cell.state
}

export function isExpression(source) {
  return /^\s*=/.exec(source)
}

export function getValue(cell) {
  if (cell.state) {
    return cell.value
  } else {
    // TODO: need to coerce
    return cell.text()
  }
}