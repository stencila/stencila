import { isNumber } from 'substance'
import { toIdentifier } from '../shared/expressionHelpers'
import { getCellLabel, qualifiedId as _qualifiedId } from '../shared/cellHelpers'

export const BROKEN_REF = '#BROKEN_REF'

export function recordTransformations(cell, dim, pos, count, affectedCells, visited) {
  affectedCells = affectedCells || new Set()
  visited = visited || new Set()
  cell.deps.forEach(s => {
    if (visited.has(s)) return
    visited.add(s)
    let start, end
    if (dim === 0) {
      start = s.startRow
      end = s.endRow
    } else {
      start = s.startCol
      end = s.endCol
    }
    let res = transformRange(start, end, pos, count)
    if (!res) return
    if (res === -1) {
      s._update = { type: 'broken' }
    } else {
      let type = (count < 0 ? 'delete' : 'insert') + (dim === 0 ? 'Rows' : 'Cols')
      s._update = {
        type,
        start: res.start,
        end: res.end
      }
    }
    affectedCells.add(s.cell)
  })
}

export function applyCellTransformations(cell) {
  let symbols = Array.from(cell.inputs).sort((a, b) => a.startPos - b.startPos)
  let source = cell._source
  let offset = 0
  for (let i = 0; i < symbols.length; i++) {
    let s = symbols[i]
    let update = s._update
    if (!update) continue
    delete s._update
    // compute derived content according to parameters
    let oldName = s.name
    let oldScope = s.scope
    let oldOrigStr = s.origStr
    let oldMangledStr = s.mangledStr
    let newName = oldName
    let newScope = oldScope
    let newOrigStr = oldOrigStr
    let newMangledStr = oldMangledStr
    switch (update.type) {
      case 'insertRows':
      case 'deleteRows': {
        s.startRow = update.start
        s.endRow = update.end
        newName = getCellSymbolName(s)
        newOrigStr = oldOrigStr.replace(oldName, newName)
        newMangledStr = oldMangledStr.replace(toIdentifier(oldName), toIdentifier(newName))
        break
      }
      case 'insertCols':
      case 'deleteCols': {
        s.startCol = update.start
        s.endCol = update.end
        newName = getCellSymbolName(s)
        newOrigStr = oldOrigStr.replace(oldName, newName)
        newMangledStr = oldMangledStr.replace(toIdentifier(oldName), toIdentifier(newName))
        break
      }
      case 'broken': {
        s.type = 'var'
        s.startRow = s.startCol = s.endRow = s.endCol = null
        newName = BROKEN_REF
        newOrigStr = BROKEN_REF
        newMangledStr = BROKEN_REF
        break
      }
      case 'rename': {
        if (oldScope) {
          newOrigStr = oldOrigStr.replace(oldScope, update.scope)
          newMangledStr = oldMangledStr.replace(toIdentifier(oldScope), toIdentifier(update.scope))
        }
        break
      }
      default:
        throw new Error('Illegal state')
    }
    let newStartPos = s.startPos + offset
    let newEndPos = newStartPos + newOrigStr.length
    let newSource = source.original.slice(0, s.startPos+offset) + newOrigStr + source.original.slice(s.endPos+offset)
    let newTranspiled = source.transpiled.slice(0, s.startPos+offset) + newMangledStr + source.transpiled.slice(s.endPos+offset)

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

export function transformRange(start, end, pos, count) {
  if (!count) return false
  if(!isNumber(pos) || !isNumber(count)) throw new Error("pos and count must be integers")
  if(end < pos) return false
  if(count > 0) {
    if(pos <= start) {
      start += count
    }
    if(pos <= end) {
      end += count
    }
  } else {
    // for removal count < 0
    count = -count
    // null means deleted
    if (start >= pos && end < pos + count) return -1
    const x1 = pos
    const x2 = pos + count
    if (x2 <= start) {
      start -= count
      end -= count
    } else {
      if (pos <= start) {
        start = start - Math.min(count, start-x1)
      }
      if (pos <= end) {
        end = end - Math.min(count, end-x1+1)
      }
    }
  }
  return { start, end }
}

export function getCellSymbolName(s) {
  let newName = getCellLabel(s.startRow, s.startCol)
  if (s.type === 'range') {
    newName += ':' + getCellLabel(s.endRow, s.endCol)
  }
  return newName
}
