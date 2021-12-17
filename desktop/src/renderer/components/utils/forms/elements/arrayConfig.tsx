import { h } from '@stencil/core'
import { JSONSchema7 } from 'json-schema'
import { ConfigSchema, FormElement } from './types'

type ArrayConfig = JSONSchema7 & {
  type: 'array'
}

export const arrayGuard = (schema: ConfigSchema): schema is ArrayConfig =>
  typeof schema !== 'boolean' && schema.type === 'array'

export const arrayInput: FormElement<ArrayConfig> = (
  schema,
  { onChangeHandler } = {}
) => {
  const items = schema.items

  if (items === undefined || typeof items === 'boolean' || Array.isArray(items))
    return []

  return [
    schema.description !== undefined && (
      <p class="helpText">{schema.description}</p>
    ),
    <textarea onChange={onChangeHandler}>{items.enum?.join('\n')}</textarea>,
    items.description !== undefined && (
      <p class="helpText">{items.description}</p>
    ),
  ]
}
