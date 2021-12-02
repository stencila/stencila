function decodeValue(json) {
  return JSON.parse(json)
}

function encodeValue(value) {
  return JSON.stringify(value)
}

function encodeError(error) {
  const codeError = { type: 'CodeError' }
  if (error.name) codeError.errorType = error.name
  if (error.message) codeError.errorMessage = error.message
  else codeError.errorMessage = error.toString()
  if (error.stack) codeError.stackTrace = error.stack
  return JSON.stringify(codeError)
}

module.exports = {
  decodeValue,
  encodeValue,
  encodeError,
}
