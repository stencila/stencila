import { h } from '@stencil/core'
import { JSONSchema7, JSONSchema7Definition } from 'json-schema'

export type ConfigSchema = JSONSchema7Definition

// Boolean config

type BooleanConfig = JSONSchema7 & { type: 'boolean'; default?: boolean }

const booleanGuard = (schema: ConfigSchema): schema is BooleanConfig => {
  return typeof schema !== 'boolean' && schema.type === 'boolean'
}

const checkbox = (label: string, schema: BooleanConfig) => {
  return (
    <div>
      <label>{label}</label>
      <input type="checkbox" checked={schema.default} />
      {schema.description && <p>{schema.description}</p>}
    </div>
  )
}

// Select config

type SelectConfig = JSONSchema7 & { type: 'string'; enum: string[] }

const selectGuard = (schema: ConfigSchema): schema is SelectConfig => {
  return typeof schema !== 'boolean' && schema.type === 'string'
}

const select = (label: string, schema: SelectConfig) => {
  return (
    <div>
      <label>{label}</label>
      <select>
        {(schema.enum ?? []).map((option) => {
          // @ts-ignore
          return <option value={option}>{option}</option>
        })}
      </select>
      {schema.description && <p>{schema.description}</p>}
    </div>
  )
}

const generic = (label: string, schema: ConfigSchema) => {
  if (typeof schema === 'boolean') return

  return (
    <div>
      <label>{label}</label>
      {schema.description && <p>{schema.description}</p>}
      <pre>
        <code>{JSON.stringify(schema, null, 2)}</code>
      </pre>
    </div>
  )
}

export const build = (schema: ConfigSchema) => {
  if (typeof schema === 'boolean') return

  return Object.entries(schema.definitions ?? {}).map(([prop, subSchema]) => {
    if (booleanGuard(subSchema)) {
      return checkbox(prop, subSchema)
    }
    if (selectGuard(subSchema)) {
      return select(prop, subSchema)
    }

    return generic(prop, subSchema)
  })
}
