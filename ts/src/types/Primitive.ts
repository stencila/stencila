// Union type for all primitives values
export type Primitive =
  null |
  boolean |
  number |
  string |
  Primitive[] |
  { [key: string]: Primitive };
