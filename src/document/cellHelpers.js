export function findMini(cell) {
  return cell.find('source-code[language=mini]')
}

export function findSource(cell) {
  return cell.find('source-code:not([language=mini])')
}
