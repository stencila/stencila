import { h } from '@stencil/core'
import { JSONSchema7 } from 'json-schema'
import { ConfigSchema } from '../formBuilder'
import { FormElement } from './types'

type StringConfig = JSONSchema7 & {
  type: 'string'
  enum: undefined
  allOf: undefined
}

export const stringGuard = (schema: ConfigSchema): schema is StringConfig =>
  (typeof schema !== 'boolean' && schema.type?.includes('string')) ?? false

export const stringInput: FormElement<StringConfig> = (
  schema,
  { onChangeHandler } = {}
) => {
  return (
    <input
      onChange={onChangeHandler}
      placeholder={schema.default?.toString()}
    />
  )
}
