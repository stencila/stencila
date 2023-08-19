// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Array } from './Array'
import { Boolean } from './Boolean'
import { Integer } from './Integer'
import { Null } from './Null'
import { Number } from './Number'
import { Object } from './Object'
import { String } from './String'
import { UnsignedInteger } from './UnsignedInteger'

// Union type for all primitives values
export type Primitive =
  Null |
  Boolean |
  Integer |
  UnsignedInteger |
  Number |
  String |
  Array |
  Object;
