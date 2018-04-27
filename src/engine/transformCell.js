import { toIdentifier } from '../shared/expressionHelpers'
import { getCellLabel, qualifiedId as _qualifiedId } from '../shared/cellHelpers'

export default function transformCell(cell, mode) {
  let symbols = Array.from(cell.inputs).sort((a, b) => a.startPos - b.startPos)
  let source = cell._source
  let offset = 0
  for (let i = 0; i < symbols.length; i++) {
    let s = symbols[i]
    let update = s._update
    if (!update) continue
    delete s._update

    // name
    let oldName = s.name
    let oldScope = s.scope
    let oldOrigStr = s.origStr
    let oldMangledStr = s.mangledStr

    let newName = oldName
    let newScope = oldScope
    let newOrigStr, newMangledStr

    if (mode === 'rows') {
      s.startRow = update.start
      s.endRow = update.end
      newName = _getCellSymbolName(s)
      newOrigStr = oldOrigStr.replace(oldName, newName)
      newMangledStr = oldMangledStr.replace(toIdentifier(oldName), toIdentifier(newName))
    } else if (mode === 'cols') {
      s.startCol = update.start
      s.endCol = update.end
      newName = _getCellSymbolName(s)
      newOrigStr = oldOrigStr.replace(oldName, newName)
      newMangledStr = oldMangledStr.replace(toIdentifier(oldName), toIdentifier(newName))
    } else { // if (mode === 'rename') {
      newOrigStr = oldOrigStr.replace(oldScope, update.scope)
      newMangledStr = oldMangledStr.replace(toIdentifier(oldScope), toIdentifier(update.scope))
    }
    // start- and endPos
    let newStartPos = s.startPos + offset
    let newEndPos = newStartPos + newOrigStr.length
    // 2. replace the symbol in the source and the transpiled source
    let newSource = source.original.slice(0, s.startPos) + newOrigStr + source.original.slice(s.endPos)
    let newTranspiled = source.transpiled.slice(0, s.startPos) + newMangledStr + source.transpiled.slice(s.endPos)

    // finally write the updated values
    s.name = newName
    s.id = _qualifiedId(s.docId, newName)
    s.scope = newScope
    s.origStr = newOrigStr
    s.mangledStr = newMangledStr
    s.startPos = newStartPos
    s.endPos = newEndPos
    source.original = newSource
    source.transpiled = newTranspiled
    source.symbolMapping[newMangledStr] = s
    delete source.symbolMapping[oldMangledStr]
    // update the offset if the source is getting longer because of this change
    // this has an effect on all subsequent symbols
    offset += newOrigStr.length - oldOrigStr.length
  }
}

function _getCellSymbolName(s) {
  let newName = getCellLabel(s.startRow, s.startCol)
  if (s.type === 'range') {
    newName += ':' + getCellLabel(s.endRow, s.endCol)
  }
  return newName
}
