import { EditorSession, EventListener, forEach } from 'substance'

const _off = EditorSession.prototype.off

/* eslint-disable no-invalid-this*/

function on(...args) {
  let self = this // eslint-disable-line no-invalid-this
  let name = args[0]
  if (self._flowStages.indexOf(name) >= 0) {
    // remove the stage name from the args
    args.shift()
    let options = args[2] || {}
    let resource = options.resource
    if (resource) {
      delete options.resource
      args.unshift(resource)
    }
    this._registerObserver(name, args)
  } else {
    EventListener.prototype.on.apply(this, arguments)
  }
}

function off() {
  let self = this // eslint-disable-line no-invalid-this
  if (arguments.length === 1) {
    _off.apply(self, arguments)
  } else {
    const stage = arguments[0]
    const method = arguments[1]
    const observer = arguments[2]
    self._deregisterObserver(stage, method, observer)
  }
}

function _deregisterObserver(stage, method, observer) {
  let self = this // eslint-disable-line no-invalid-this
  if (arguments.length === 1) {
    // TODO: we should optimize this, as ATM this needs to traverse
    // a lot of registered listeners
    forEach(self._observers, (observers) => {
      for (let i = observers.length-1; i >=0 ; i--) {
        const o = observers[i]
        if (o.context === observer) {
          observers.splice(i, 1)
          o._deregistered = true
        }
      }
    })
  } else {
    let observers = self._observers[stage]
    // if no observers are registered, then this might not
    // be a deregistration for a stage, but a regular event
    if (!observers) {
      EventListener.prototype.off.apply(self, arguments)
    } else {
      for (let i = observers.length-1; i >= 0; i--) {
        let o = observers[i]
        if (o.handler === method && o.context === observer) {
          observers.splice(i, 1)
          o._deregistered = true
        }
      }
    }
  }
}

EditorSession.prototype.on = on
EditorSession.prototype.off = off
EditorSession.prototype._deregisterObserver = _deregisterObserver
