// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { ArrayValidator } from "./ArrayValidator.js";
import { BooleanValidator } from "./BooleanValidator.js";
import { ConstantValidator } from "./ConstantValidator.js";
import { DateTimeValidator } from "./DateTimeValidator.js";
import { DateValidator } from "./DateValidator.js";
import { DurationValidator } from "./DurationValidator.js";
import { EnumValidator } from "./EnumValidator.js";
import { IntegerValidator } from "./IntegerValidator.js";
import { NumberValidator } from "./NumberValidator.js";
import { StringValidator } from "./StringValidator.js";
import { TimeValidator } from "./TimeValidator.js";
import { TimestampValidator } from "./TimestampValidator.js";
import { TupleValidator } from "./TupleValidator.js";

/**
 * Union type for validators.
 */
export type Validator =
  ArrayValidator |
  BooleanValidator |
  ConstantValidator |
  DateTimeValidator |
  DateValidator |
  DurationValidator |
  EnumValidator |
  IntegerValidator |
  NumberValidator |
  StringValidator |
  TimeValidator |
  TimestampValidator |
  TupleValidator;

/**
 * Create a `Validator` from an object
 */
export function validator(other: Validator): Validator {
  switch(other.type) {
    case "ArrayValidator":
    case "BooleanValidator":
    case "ConstantValidator":
    case "DateTimeValidator":
    case "DateValidator":
    case "DurationValidator":
    case "EnumValidator":
    case "IntegerValidator":
    case "NumberValidator":
    case "StringValidator":
    case "TimeValidator":
    case "TimestampValidator":
    case "TupleValidator":
      return hydrate(other) as Validator
    default:
      throw new Error(`Unexpected type for Validator: ${other.type}`);
  }
}
