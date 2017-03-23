import _set from 'lodash/set'

export default function update(object, path, value) {
  return _set(object, path, value)
}
