/*
  A graph allowing to define dependencies
  between symbols.

  For example, for an Expression engine a topologically correct
  evaluation order can be derived by ordering the expressions
  by the rank of their input dependencies.

  The `rank` of a variable denotes the number of reduction steps
  necessary to retrieve the value of a variable.
  This can be used to implement static scheduling strategies.

*/
export default class CellGraph {

  constructor(context) {
    this._context = context

    // graph's source containing the dependency formulated
    // in terms of symbol names
    // e.g. { cell1: { id: 'cell1', inputs: ['x', 'y'], output: 'z' }}
    this._cells = {}

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

  _compile() {
    let ids = Object.keys(this._cells)
    // 1. Create a mapping from symbol name to cell
    let createdBy = {}
    ids.forEach((id) => {
      let cell = this._cells[id]
      let output = cell.output
      if (output) {
        if (createdBy[output]) {
          throw new Error(`Multiple cells create the same output: ${output}`)
        }
        createdBy[output] = cell
      }
    })
    this._createdBy = createdBy

    // 2. Create backward graph i.e. in-going edges
    let ins = {}
    ids.forEach((id) => {
      let cell = this._cells[id]
      let inputs = cell.inputs.map((symbol) => {
        let input = this._resolve(symbol)
        if (!input) {
          console.error('TODO: store an error message')
        }
        return input
      }).filter(Boolean)
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
        let outputs = outs[input.id]
        if (!outputs) {
          outputs = new Set()
          outs[input.id] = outputs
        }
        outputs.add(cell)
      })
    })
    this._outs = outs

    let ranks = {}
    ids.forEach((id) => {
      // HACK: transforming outs into an array making it easier to work with
      if (outs[id]) {
        outs[id] = Array.from(outs[id])
      }
      this._computeRank(id, this.getInputs(id), ranks)
    })
    this._ranks = ranks
  }

  _resolve(symbol) {
    // TODO: support complex symbols
    let res = this._createdBy[symbol]
    // TODO: we would need to use a context to lookup
    // other things not being an evaluated cell, e.g. spreadsheet cells with
    // constant values
    // if (res) return res
    // return this._context.lookup(symbol)
    return res
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