import { forEach } from 'substance'
/*
  Dependency graph for Stencila Cell Engine.
*/
export default class CellGraph {

  constructor() {

    // store for cells containing expressions
    this._cells = {}

    // documents get registered via their name
    // so that we can lookup cells and inputs via
    // references such as 'sheet1.C1' or 'sheet1.A1:B10'
    this._documents = {}

    // which cell is producing a symbol
    this._createdBy = {}

    // which cells is a cell depending on
    this._ins = {}

    // which cells are depending on the output of a cell
    this._outs = {}

    // cell ranks denoting the level of dependencies
    this._ranks = null
  }

  contains(cellId) {
    return Boolean(this._cells[cellId])
  }

  getCell(cellId) {
    return this._cells[cellId]
  }

  getInputs(cellId) {
    return this._ins[cellId] || []
  }

  getOutputs(cellId) {
    return this._outs[cellId] || []
  }

  registerDocument(id, doc) {
    this._documents[id] = doc
  }

  getDocument(id) {
    return this._documents[id]
  }

  addCell(cell) {
    this._cells[cell.id] = cell
  }

  updateCell(cell) {
    this._cells[cell.id] = cell
    this._compile()
  }

  removeCell(id) {
    delete this._cells[id]
    this._compile()
  }


  lookup(symbol) {
    switch(symbol.type) {
      case 'var': {
        return this._resolve(symbol)[0]
      }
      case 'cell': {
        let sheet = this._documents[symbol.docId]
        return sheet.getCell(symbol.row, symbol.col)
      }
      case 'range': {
        // TODO: lookup all cells and then
        // reduce to either a table, an array, or a value
        let sheet = this._documents[symbol.docId]
        let { startRow, endRow, startCol, endCol } = symbol
        if (startRow > endRow) {
          ([startRow, endRow] = [endRow, startRow])
        }
        if (startCol > endCol) {
          ([startCol, endCol] = [endCol, startCol])
        }
        let matrix = []
        for (let i = startRow; i <= endRow; i++) {
          let row = []
          for (let j = startCol; j <= endCol; j++) {
            row.push(sheet.getCell(i, j))
          }
          matrix.push(row)
        }
        return matrix
      }
      default:
        throw new Error('Invalid state')
    }
  }

  _compile() {
    let ids = Object.keys(this._cells)
    // 1. Create a mapping from symbol name to cell
    let createdBy = {}
    ids.forEach((id) => {
      let cell = this._cells[id]
      let output = cell.output
      let docId = cell.docId
      if (output) {
        let varId = `${docId}.${output}`
        if (createdBy[varId]) {
          throw new Error(`Multiple cells create the same output: ${output}`)
        }
        createdBy[varId] = cell
      }
    })
    this._createdBy = createdBy

    // 2. Create backward graph i.e. in-going edges
    let ins = {}
    ids.forEach((id) => {
      let cell = this._cells[id]
      let inputs = []
      // TODO: handle lookup errors
      cell.inputs.forEach((symbol) => {
        switch(symbol.type) {
          case 'var': {
            let cell = this.lookup(symbol)
            inputs.push(cell)
            break
          }
          case 'cell': {
            let cell = this.lookup(symbol)
            inputs.push(cell)
            break
          }
          case 'range': {
            let matrix = this.lookup(symbol)
            for (let i = 0; i < matrix.length; i++) {
              inputs = inputs.concat(matrix[i])
            }
            break
          }
          default:
            throw new Error('FIXME: invalid state')
        }
      })
      // HACK: for now we strip all unresolved symbols
      inputs = inputs.filter(Boolean)
      ins[cell.id] = inputs
    })
    this._ins = ins

    // 3. Compute a forward graph i.e. out-going edges
    let outs = {}
    ids.forEach((id) => {
      let inputs = ins[id]
      let cell = this._cells[id]
      inputs.forEach((input) => {
        // Note: this should have been reported before
        if (!input) return
        // FIXME: need to avoid this confusion with adapter and cell
        let adapter = input // input._adapter
        // HACK
        if (input._adapter) adapter = input._adapter
        let outputs = outs[adapter.id]
        if (!outputs) {
          outputs = new Set()
          outs[adapter.id] = outputs
        }
        outputs.add(cell)
      })
    })
    this._outs = outs

    // HACK: transforming outs into an array making it easier to work with
    forEach(outs, (cells, id) => {
      outs[id] = Array.from(cells)
    })

    let ranks = {}
    ids.forEach((id) => {
      this._computeRank(id, this.getInputs(id), ranks)
    })
    this._ranks = ranks
  }

  _resolve(symbol) {
    switch(symbol.type) {
      case 'var': {
        let id = `${symbol.docId}.${symbol.name}`
        return [this._createdBy[id]]
      }
      case 'cell': {
        let sheet = this._documents[symbol.docId]
        if (!sheet) {
          // TODO: return this error
          console.error('Could find sheet with name', symbol.scope)
          return undefined
        }
        let cell = sheet.getCell(symbol.row, symbol.col)
        return [cell.id]
      }
      case 'range': {
        let ids = []
        let sheet = this._documents[symbol.docId]
        if (!sheet) {
          // TODO: return this error
          console.error('Could find sheet with name', symbol.scope)
          return undefined
        }
        for (let i = symbol.startRow; i <= symbol.endRow; i++) {
          for (let j = symbol.startCol; j <= symbol.endCol; j++) {
            let cell = sheet.getCell(i, j)
            ids.push(cell.id)
          }
        }
        return ids
      }
      default:
        throw new Error('Invalid state')
    }
  }

  _computeRank(id, inputs, ranks) {
    let rank
    // dependencies might have been computed already
    // when this entry has been visited through the dependencies
    // of another entry
    // Initially, we set level=-1, so when we visit
    // an entry with level===-1, we know that there
    // must be a cyclic dependency.
    if (ranks.hasOwnProperty(id)) {
      rank = ranks[id]
      if (rank === -1) {
        throw new Error('Found a cyclic dependency.')
      }
      return rank
    }
    // using value -1 as guard to detect cyclic deps
    ranks[id] = -1
    // a resource without dependencies has rank = 0
    rank = 0
    if (inputs.length > 0) {
      let depRanks = inputs.map((cell) => {
        return this._computeRank(cell.id, this.getInputs(cell.id), ranks)
      })
      rank = Math.max(...depRanks) + 1
    }
    ranks[id] = rank
    return rank
  }
}