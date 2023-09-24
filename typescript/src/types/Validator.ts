// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { ArrayValidator } from './ArrayValidator'
import { BooleanValidator } from './BooleanValidator'
import { ConstantValidator } from './ConstantValidator'
import { DateTimeValidator } from './DateTimeValidator'
import { DateValidator } from './DateValidator'
import { DurationValidator } from './DurationValidator'
import { EnumValidator } from './EnumValidator'
import { IntegerValidator } from './IntegerValidator'
import { NumberValidator } from './NumberValidator'
import { StringValidator } from './StringValidator'
import { TimeValidator } from './TimeValidator'
import { TimestampValidator } from './TimestampValidator'
import { TupleValidator } from './TupleValidator'

// Union type for validators.
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
    default: throw new Error(`Unexpected type for Validator: ${other.type}`)
  }
}
