export function decodeValue(json: string) {
  return JSON.parse(json)[0]
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
