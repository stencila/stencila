import { h, VNode } from '@stencil/core'
import { JSONSchema7 } from 'json-schema'
import { ConfigSchema } from '../formBuilder'
import { FormElement } from './types'

type SelectConfig = JSONSchema7 & { type: 'string'; enum: string[] }

export const selectGuard = (schema: ConfigSchema): schema is SelectConfig =>
  typeof schema !== 'boolean' &&
  (Array.isArray(schema.enum) || Array.isArray(schema.allOf))

export const select: FormElement<SelectConfig> = (
  schema: SelectConfig,
  { onChangeHandler } = {}
) => (
  <select onChange={onChangeHandler}>
    {(schema.enum ?? []).reduce((options: VNode[], option) => {
      return option === null ||
        typeof option === 'boolean' ||
        typeof option === 'object'
        ? options
        : [...options, <option value={option}>{option}</option>]
    }, [])}
  </select>
)
