import { h } from '@stencil/core'
import { JSONSchema7 } from 'json-schema'
import { ConfigSchema } from '../formBuilder'
import { FormElement } from './types'

type BooleanConfig = JSONSchema7 & { type: 'boolean'; default?: boolean }

export const booleanGuard = (schema: ConfigSchema): schema is BooleanConfig =>
  typeof schema !== 'boolean' && schema.type === 'boolean'

export const checkbox: FormElement<BooleanConfig> = (
  schema: BooleanConfig,
  { onChangeHandler } = {}
) => (
  <input onChange={onChangeHandler} type="checkbox" checked={schema.default} />
)
