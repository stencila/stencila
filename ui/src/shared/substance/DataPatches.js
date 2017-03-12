import {Data} from 'substance'

// Path based implementation in model/data/Data.js was not sufficient
// This proposal tries to generalize it.
Data.prototype._set = function(path, newValue) {
  let oldValue = _setValue(this.nodes, path, newValue)
  return oldValue
}

function _setValue(root, path, newValue) {
  let ctx = root
  let L = path.length
  for (let i = 0; i < L-1; i++) {
    ctx = ctx[path[i]]
    if (!ctx) throw new Error('Can not set value.')
  }
  let oldValue = ctx[path[L-1]]
  ctx[path[L-1]] = newValue
  return oldValue
}
