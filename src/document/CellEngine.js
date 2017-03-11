import { forEach } from 'substance'
import { Engine } from 'substance-mini'
import { unpack } from 'stencila-js'

export default
class CellEngine extends Engine {

  constructor(editorSession) {
    super()

    this.editorSession = editorSession
    this.doc = editorSession.getDocument()

    this._cells = {}
    this._contexts = editorSession.getContext().stencilaContexts || {}

    console.log('INITIALIZING CELL ENGINE')
    this._initialize()

    editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })
  }

  dispose() {
    super.dispos()
    this.editorSession.off(this)
  }

  /*
    Calling into the context.

    There are different types of calls:
    - function calls: the arguments are positional, and passed
      to the external function
    - external cells: arguments are provided as an object with
      names taken from the signature. The context is used to
      execute the sourceCode, using the arguments object.
    - chunk: like with external cells, arguments are provided
      as object. The source code is run in the same way as we know
      it from notebook cells, such as in Jupyter.
  */
  callFunction(funcNode) {
    const functionName = funcNode.name
    if (functionName === 'call' || functionName === 'run') {
      const expr = funcNode.expr
      const cell = expr._cell
      if (!cell) throw new Error('Internal error: no cell associated with expression.')
      if(!cell.language) throw new Error('language is mandatory for "call"')
      const lang = cell.language
      const context = this._contexts[lang]
      if (!context) throw new Error('No context for language ' + lang)
      const sourceCode = cell.sourceCode || ''
      const options = { pack: lang === 'js' ? false : true }
      const args = {}
      funcNode.forEach((arg) => {
        const name = arg.name
        if (!name) {
          console.warn('Only variables can be used with chunks and external cells')
          return
        }
        args[name] = arg.getValue()
      })
      return _unwrapResult(
        context.call(sourceCode, args, options),
        options
      )
    }

    // regular function calls: we need to lookup
    const func = this._lookupFunction(functionName)
    if (func) {
      // TODO: if we had the functions signature
      // we could support keyword arguments here
      const args = funcNode.args.map(arg => arg.getValue())
      const { context, contextName } = func
      const options = { pack: contextName === 'js' ? false : true }
      return _unwrapResult(
        context.callFunction(functionName, args, options),
        options
      )
    } else {
      return Promise.reject(`Could not resolve function "${functionName}"`)
    }
  }

  _lookupFunction(functionName) {
    const contexts = this._contexts
    let names = Object.keys(contexts)
    for (let i = 0; i < names.length; i++) {
      const contextName = names[i]
      const context = contexts[contextName]
      if (context.hasFunction(functionName)) {
        return { contextName, context }
      }
    }
  }

  _onDocumentChange(change) {
    const doc = this.doc
    let needsUpdate = false
    // HACK: exploiting knowledge about ops used for manipulating cells
    // - create/delete of cells
    forEach(change.deleted, (node) => {
      if (node.type === 'cell') {
        this._deregisterCell(node.id)
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
    let cells = this.doc.getIndex('type').get('cell')
    forEach(cells, (cell) => {
      this._registerCell(cell)
    })
    // this updates the dependency graph and triggers evaluation
    super.update()
  }

  _registerCell(cell) {
    this._cells[cell.id] = cell
    if (cell._expr) {
      this._addExpression(cell._expr)
      cell.on('expression:changed', this._updateCell, this)
    } else {
      console.error(cell.error)
    }
    this.emit('engine:updated')
    return cell
  }

  _deregisterCell(cell) {
    cell.off(this)
    delete this._cells[cell.id]
    this._removeExpression(cell.id)
  }

  _updateCell(cell) {
    this._removeExpression(cell.id)
    if (cell._expr) {
      this._addExpression(cell._expr)
    }
  }

}

function _unwrapResult(p, options) {
  const pack = options.pack !== false
  return new Promise((resolve, reject) => {
    p.then((res) => {
      if (res.errors) {
        reject(res.errors)
      } else {
        const output = pack ? unpack(res.output) : res.output
        resolve(output)
      }
    })
  })
}

