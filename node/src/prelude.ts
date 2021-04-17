export function toJSON(value: unknown): string {
  return JSON.stringify(value)
}

export function fromJSON<Type>(json: string): Type {
  return JSON.parse(json) as Type
}
