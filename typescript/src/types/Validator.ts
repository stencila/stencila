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
