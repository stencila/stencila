import { h } from '@stencil/core'
import { ConfigSchema, FormElement } from './types'

// Fallback renderer for unsupported configuration types, usually JSON objects or key:value pairs
export const generic: FormElement<ConfigSchema> = (
  schema,
  { onChangeHandler } = {}
) => {
  if (typeof schema === 'boolean') return

  return <textarea onChange={onChangeHandler}></textarea>
}
