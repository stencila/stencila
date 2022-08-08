export function decodeValue(value: any) {
  return value
}

export function encodeValue(value: string) {
  return JSON.stringify(value)
}

export function encodeError(error: any) {
  const codeError = { type: 'CodeError' } as any
  if (error.name) codeError.errorType = error.name
  if (error.message) codeError.errorMessage = error.message
  if (error.stack) codeError.stackTrace = error.stack
  return JSON.stringify(codeError)
}
