import { h } from '@stencil/core'
import { JSONSchema7 } from 'json-schema'
import { ConfigSchema } from '../formBuilder'
import { FormElement } from './types'

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

  if (!items || typeof items === 'boolean' || Array.isArray(items)) return []

  return [
    schema.description && <p class="helpText">{schema.description}</p>,
    <textarea onChange={onChangeHandler}>{items.enum?.join('\n')}</textarea>,
    items.description && <p class="helpText">{items.description}</p>,
  ]
}
