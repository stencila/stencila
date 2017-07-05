import { forEach } from 'substance'
import { Engine } from 'substance-mini'
import { pack, unpack } from '../value'
import JsContext from '../js-context/JsContext'

export default
class CellEngine extends Engine {

  constructor(editorSession, options = {}) {
    super(Object.assign({
      waitForIdle: 500
    }, options))

    this.editorSession = editorSession
    this.doc = editorSession.getDocument()
    this.host = editorSession.getContext().host

    this._cells = {}
    this._inputs = {}

    // TODO : temporaray, remove!
    this._contexts = {
      'js': new JsContext()
    }

    // console.log('INITIALIZING CELL ENGINE')
    forEach(this.doc.getNodes(), (node) => {
      switch(node.type) {
        case 'cell':
        case 'inline-cell': {
          this._registerCell(node)
          break
        }
        case 'select':
        case 'range-input': {
          this._registerInput(node)
          break
        }
        default:
          //
      }
    })

    editorSession.on('render', this._onDocumentChange, this, {
      resource: 'document'
    })

    // this updates the dependency graph and triggers evaluation
    super.update()
  }

  dispose() {
    super.dispose()

    forEach(this._cells, (cell) => {
      cell.off(this)
    })
    forEach(this._inputs, (input) => {
      input.off(this)
    })
    this.editorSession.off(this)

    this._cells = {}
    this._inputs = {}
  }

  /*
    Calling into the context.

    There are different types of calls:
    - function calls: the arguments are positional (ATM) and passed as array
    - external cells: arguments are passed as object with
      names taken from the signature. The context is used to
      execute the sourceCode, using the arguments object.
    - chunk: like with external cells, arguments are provided
      as object. The source code is run in the same way as we know
      it from notebook cells, such as in Jupyter.
  */
  callFunction(funcNode) {
    const functionName = funcNode.name
    const expr = funcNode.expr
    const cell = expr._cell
    if (!cell) {
      throw new Error('Internal error: no cell associated with expression.')
    }
    switch(functionName) {
      // `call` (a context function scope) or `run` (context's globals scope) external code
      case 'call':
      case 'run': {
        if(!cell.context) {
          funcNode.addErrors([{
            message: 'Calls to external code must have "context" set.'
          }])
          return
        }
        return this._getContext(cell.context).then(context => {
          if (!context) {
            funcNode.addErrors([{
              line: 0, column: 0,
              message: `No context found matching "${cell.context}"`
            }])
          } else {
            let packing = !(context instanceof JsContext)
            const options = { pack: packing }
            const sourceCode = cell.sourceCode || ''
            if (functionName === 'call') {
              // Convert all arguments into an object
              const args = {}
              // Unnamed argunents are expected to be variables and the variable name is
              // used as the name of the argument
              funcNode.args.forEach((arg) => {
                if (arg.type !== 'var') {
                  funcNode.addErrors([{
                    message: 'Calls to external code must use variables or named arguments'
                  }])
                  return
                }
                let value = arg.getValue()
                args[arg.name] = packing ? pack(value) : value
              })
              // For named arguments, just use the name and the value
              funcNode.namedArgs.forEach((arg) => {
                let value = arg.getValue()
                args[arg.name] = packing ? pack(value) : value
              })
              return _unwrapResult(
                funcNode,
                context.callCode(sourceCode, args, options),
                options
              )
            } else {
              return _unwrapResult(
                funcNode,
                context.runCode(sourceCode, options),
                options
              )
            }
          }
        })
      }
      // execute an external function
      default: {
        // regular function calls: we need to lookup
        const func = this._lookupFunction(functionName)
        if (func) {
          const { context, contextName } = func
          let packing = contextName === 'js' ? false : true
          const options = { pack: packing }
          // Unnamed arguments are expected to be variables and the variable name is
          // used as the name of the argument
          let args = []
          if (funcNode.args) {
            args = funcNode.args.map((arg) => {
              let value = arg.getValue()
              return packing ? pack(value) : value
            })
          }
          // For named arguments, just use the name and the value
          let namedArgs = {}
          if (funcNode.namedArgs) {
            for (let arg of funcNode.namedArgs) {
              let value = arg.getValue()
              namedArgs[arg.name] = packing ? pack(value) : value
            }
          }
          return _unwrapResult(
            funcNode,
            context.callFunction(functionName, args, namedArgs, options),
            options
          )
        } else {
          let msg = `Could not resolve function "${functionName}"`
          // Note: we just return undefined and add a runtime error
          funcNode.addErrors([{
            message: msg
          }])
          return
        }
      }
    }
  }

  _getContext(name) {
    // Attempt to get context from those already known to this
    // host using it's name
    return this.host.get(`name://${name}`).then(context => {
      if (context) return context
      else {
        // Determine the type of context from the context name
        let match = name.match(/([a-zA-Z]+)([\d_].+)?/)
        if (match) {
          let type = match[1]
          return this.host.post(type, name).then(context => {
            return context
          }).catch(() => {
            return null
          })
        } else {
          return null
        }
      }
    })
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
      switch (node.type) {
        case 'cell':
        case 'inline-cell': {
          this._deregisterCell(node.id)
          needsUpdate = true
          break
        }
        case 'select':
        case 'range-input': {
          this._deregisterInput(node.id)
          needsUpdate = true
          break
        }
        default:
          //
      }
    })
    forEach(change.created, (node) => {
      switch (node.type) {
        case 'cell':
        case 'inline-cell': {
          this._registerCell(doc.get(node.id))
          needsUpdate = true
          break
        }
        case 'select':
        case 'range-input': {
          this._registerInput(doc.get(node.id))
          break
        }
        default:
          //
      }
    })

    if (needsUpdate) super.update()
  }

  _registerCell(cell) {
    this._cells[cell.id] = cell
    if (cell.errors && cell.errors.length) {
      console.error(cell.error)
    } else {
      if (cell._expr) {
        this._addExpression(cell._expr)
      }
      cell.on('expression:updated', this._updateCell, this)
      this.emit('engine:updated')
    }
    return cell
  }

  _deregisterCell(cellId) {
    const cell = this._cells[cellId]
    if (cell) {
      cell.off(this)
      delete this._cells[cell.id]
      this._removeExpression(cell.id)
    }
  }

  _updateCell(cell) {
    // console.log('### Updating cell', cell.id)
    this._removeExpression(cell.id)
    if (cell._expr) {
      this._addExpression(cell._expr)
    }
    super.update()
  }

  _registerInput(input) {
    this._inputs[input.id] = input
    const name = input.name
    if (name) {
      this.setValue(name, input.value)
    }
    input.on('name:updated', this._onInputNameChanged, this)
    input.on('value:updated', this._onInputValueChanged, this)
  }

  _deregisterInput(inputId) {
    const input = this._inputs[inputId]
    if (input) {
      input.off(this)
      delete this._inputs[input.id]
      const name = input.name
      if (name) {
        this.setValue(name, undefined)
      }
    }
  }

  _onInputNameChanged(input, oldName) {
    const newName = input.name
    if (oldName) {
      this.setValue(oldName, undefined)
    }
    if (newName) {
      this.setValue(newName, input.value, 'propagate-immediately')
    }
  }

  _onInputValueChanged(input) {
    const name = input.name
    if (name) {
      this.setValue(name, input.value, 'propagate-immediately')
    }
  }
}

function _unwrapResult(funcNode, p, options) {
  const pack = options.pack !== false
  return p.then((res) => {
    if (res.errors) {
      funcNode.addErrors(res.errors)
      return undefined
    }
    if (res.output) {
      const output = pack ? unpack(res.output) : res.output
      return output
    }
  })
}
