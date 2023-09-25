// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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
    case "ArrayValidator": return ArrayValidator.from(other as ArrayValidator);
    case "BooleanValidator": return BooleanValidator.from(other as BooleanValidator);
    case "ConstantValidator": return ConstantValidator.from(other as ConstantValidator);
    case "DateTimeValidator": return DateTimeValidator.from(other as DateTimeValidator);
    case "DateValidator": return DateValidator.from(other as DateValidator);
    case "DurationValidator": return DurationValidator.from(other as DurationValidator);
    case "EnumValidator": return EnumValidator.from(other as EnumValidator);
    case "IntegerValidator": return IntegerValidator.from(other as IntegerValidator);
    case "NumberValidator": return NumberValidator.from(other as NumberValidator);
    case "StringValidator": return StringValidator.from(other as StringValidator);
    case "TimeValidator": return TimeValidator.from(other as TimeValidator);
    case "TimestampValidator": return TimestampValidator.from(other as TimestampValidator);
    case "TupleValidator": return TupleValidator.from(other as TupleValidator);
    default: throw new Error(`Unexpected type for Validator: ${other.type}`);
  }
}
