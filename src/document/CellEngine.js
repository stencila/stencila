import { forEach } from 'substance'
import { Engine } from 'substance-mini'

export default
class CellEngine extends Engine {

  constructor(editorSession) {
    super()

    this.editorSession = editorSession
    this.doc = editorSession.getDocument()

    // EXPERIMENTAL: dunno yet how we want to control
    // function lookup
    // ATM: we just iterate over the
    this._contexts = editorSession.getContext().stencilaContexts || {}

    this._initialize()

    editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
  }

  dispose() {
    this.editorSession.off(this)
  }

  callFunction(name, args) {
    let contexts = Object.values(this._contexts)
    for (let i = 0; i < contexts.length; i++) {
      const context = contexts[i]
      if (context.hasFunction(name)) {
        let res = context.callFunction(name, args)
        if (res && res.then) {
          return res
        } else {
          return Promise.resolve(res)
        }
      }
    }
    return Promise.reject(`Could not resolve function "${name}"`)
  }

  _onDocumentChange(change) {
    const doc = this.doc
    let needsUpdate = false
    // HACK: exploiting knowledge about ops used for manipulating cells
    // - create/delete of cells
    forEach(change.deleted, (node) => {
      if (node.type === 'cell') {
        this._removeExpression(node.id)
        needsUpdate = true
      }
    })

    forEach(change.created, (node) => {
      if (node.type === 'cell') {
        this._updateCell(doc.get(node.id))
        needsUpdate = true
      }
    })

    // update cells where expression or sourceCode has changed
    change.ops.forEach((op) => {
      if (op.type === 'set') {
        const id = op.path[0]
        const node = doc.get(id)
        if (node.type !== 'cell') return
        let prop = op.path[1]
        switch(prop) {
          case 'expression': {
            this._updateCell(doc.get(id))
            needsUpdate = true
            break
          }
          case 'sourceCode': {
            // TODO: we need to provide the expression with
            // the updated sourceCode and then retrigger evaluation
            break
          }
          default:
            //
        }
      }
    })

    if (needsUpdate) {
      super.update()
    }
  }

  _initialize() {
    // TODO: go over all Cells and register them with this engine
    let cells = this.doc.getIndex('type').get('cell')
    forEach(cells, (cell) => {
      this._updateCell(cell)
    })
    // this updates the dependency graph and
    // triggers evaluation
    super.update()
  }

  _updateCell(cell) {
    let oldEntry = this._entries[cell.id]
    if (oldEntry) {
      this._removeExpression(cell.id)
    }
    let parsedExpression = cell.getParsedExpression()
    if (!parsedExpression) return
    let entry = this._addExpression(parsedExpression)
    // adapter between expression node and stencila cell
    entry.on('value:updated', () => {
      cell.setValue(entry.getValue())
    })
  }

}
