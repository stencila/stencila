// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Boolean } from './Boolean';
import { Integer } from './Integer';
import { String } from './String';
import { Validator } from './Validator';

// A validator specifying constraints on an array node.
export class ArrayValidator {
  // The type of this item
  type = "ArrayValidator";

  // The identifier for this item
  id?: String;

  // Whether items can have the value `Node::Null`
  itemsNullable?: Boolean;

  // Another validator node specifying the constraints on all items in the array.
  itemsValidator?: Validator;

  // An array node is valid if at least one of its items is valid against the `contains` schema.
  contains?: Validator;

  // An array node is valid if its size is greater than, or equal to, this value.
  minItems?: Integer;

  // An array node is valid if its size is less than, or equal to, this value.
  maxItems?: Integer;

  // A flag to indicate that each value in the array should be unique.
  uniqueItems?: Boolean;

  constructor(options?: ArrayValidator) {
    if (options) Object.assign(this, options)
    
  }
}
