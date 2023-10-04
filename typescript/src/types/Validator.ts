// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type ArrayValidator } from "./ArrayValidator.js";
import { type BooleanValidator } from "./BooleanValidator.js";
import { type ConstantValidator } from "./ConstantValidator.js";
import { type DateTimeValidator } from "./DateTimeValidator.js";
import { type DateValidator } from "./DateValidator.js";
import { type DurationValidator } from "./DurationValidator.js";
import { type EnumValidator } from "./EnumValidator.js";
import { type IntegerValidator } from "./IntegerValidator.js";
import { type NumberValidator } from "./NumberValidator.js";
import { type StringValidator } from "./StringValidator.js";
import { type TimeValidator } from "./TimeValidator.js";
import { type TimestampValidator } from "./TimestampValidator.js";
import { type TupleValidator } from "./TupleValidator.js";

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
