export function getCellState(cell) {
  return cell.state
}

export function isExpression(source) {
  return /^\s*=/.exec(source)
}