import { isArray } from 'substance'
import { toString as cellStatusToString } from '../../src/engine/CellStates'
import { parseSymbol } from '../../src/shared/expressionHelpers'
import { getIndexesFromRange, getRangeFromMatrix, getRowCol } from '../../src/shared/cellHelpers'

export function getValue(cell) {
  if (cell.value) {
    return cell.value.data
  }
}

export function getValues(cells) {
  return cells.map(rowOrCell => {
    if (isArray(rowOrCell)) {
      return rowOrCell.map(getValue)
    } else {
      return getValue(rowOrCell)
    }
  })
}

export function getSource(cell) {
  return cell.source
}

export function getSources(cells) {
  return cells.map(rowOrCell => {
    if (isArray(rowOrCell)) {
      return rowOrCell.map(getSource)
    } else {
      return getSource(rowOrCell)
    }
  })
}

export function getErrors(cells) {
  return cells.map(cell => {
    return cell.errors.map(err => {
      return err.name || 'unknown'
    })
  })
}

export function getStates(cells) {
  return cells.map(cell => {
    return cellStatusToString(cell.status)
  })
}

export function queryValues(engine, expr) {
  let symbol = parseSymbol(expr)
  if (!symbol.scope) throw new Error('query must use fully qualified identifiers')
  let docId = engine._lookupDocumentId(symbol.scope)
  if (!docId) throw new Error('Unknown resource:', symbol.scope)
  switch (symbol.type) {
    case 'var': {
      return engine._graph.getValue(expr)
    }
    case 'cell': {
      let sheet = engine._docs[docId]
      let [row, col] = getRowCol(symbol.name)
      return getValue(sheet.cells[row][col])
    }
    case 'range': {
      let sheet = engine._docs[docId]
      const { startRow, startCol, endRow, endCol } = getIndexesFromRange(symbol.anchor, symbol.focus)
      let cells = getRangeFromMatrix(sheet.getCells(), startRow, startCol, endRow, endCol)
      return getValues(cells)
    }
    default:
      //
  }
}

/*
  Waits for all actions to be finished.
  This is the slowest kind of scheduling, as every cycle
  takes as long as the longest evaluation.
  In a real environment, the Engine should be triggered as often as possible,
  but still with a little delay, so that all 'simultanous' actions can be
  done at once.
*/
export function cycle(engine) {
  let actions = engine.cycle()
  return Promise.all(actions)
}

/*
  Triggers a cycle as long as next actions are coming in.
*/
export function play(engine) {
  return new Promise((resolve) => {
    function step() {
      if (_needsUpdate(engine)) {
        cycle(engine).then(step)
      } else {
        resolve()
      }
    }
    step()
  })
}

function _needsUpdate(engine) {
  const graph = engine._graph
  if (graph.needsUpdate()) return true
  const nextActions = engine._nextActions
  if (nextActions.size === 0) return false
  // update is required if there is an action that has not been suspended
  for (let [, a] of nextActions) {
    if (!a.suspended) return true
  }
  return false
}
