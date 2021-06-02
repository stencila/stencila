import { h, VNode } from '@stencil/core'
import { JSONSchema7Definition } from 'json-schema'
import { capitalize } from '../stringUtils'
import { arrayGuard, arrayInput } from './elements/arrayConfig'
import { booleanGuard, checkbox } from './elements/booleanConfig'
import { generic } from './elements/genericConfig'
import { objectGuard } from './elements/objectConfig'
import { select, selectGuard } from './elements/selectConfig'
import { stringGuard, stringInput } from './elements/stringConfig'
import { FormElementOptions } from './elements/types'

export type ConfigSchema = JSONSchema7Definition

export const changeHandler = (e: Event): void => {
  e.preventDefault()

  // @ts-ignore
  console.log(e, e.target?.value)
  // TODO: Persist changed value to the user's config file
}

const renderer = (schema: ConfigSchema, key?: string, level = 2) => {
  if (typeof schema === 'boolean') return []

  const title = schema.title ?? key ?? ''
  const HeadingLevel = `h${level}`

  const options: FormElementOptions = {
    label: title,
    onChangeHandler: changeHandler,
  }

  let field: VNode | VNode[]

  if (booleanGuard(schema)) {
    field = checkbox(schema, options)
  } else if (selectGuard(schema)) {
    field = select(schema, options)
  } else if (stringGuard(schema)) {
    field = stringInput(schema, options)
  } else if (arrayGuard(schema)) {
    field = arrayInput(schema, options)
  } else if (objectGuard(schema)) {
    field = build(schema, level + 1)
  } else {
    field = generic(schema, options)
  }

  return [
    schema.properties ? (
      <HeadingLevel>{title}</HeadingLevel>
    ) : (
      <label>{schema.title ?? key}</label>
    ),
    schema.properties && schema.description && (
      <p class="helpText">{schema.description}</p>
    ),
    field,
    !schema.properties && <p class="helpText">{schema.description}</p>,
  ]
}

export const build = (schema: ConfigSchema, level = 2): VNode | VNode[] => {
  if (typeof schema === 'boolean') return []

  if (schema.type === 'object' && !schema.properties) {
    return generic(schema)
  }

  return Object.entries(schema.properties ?? {}).reduce(
    (form: VNode[], [prop, subSchema]) => {
      const formSection: VNode | VNode[] =
        typeof subSchema !== 'boolean' && subSchema.allOf
          ? [
              subSchema.allOf.map((s) => renderer(s, capitalize(prop), level)),
              subSchema.description && (
                <p class="helpText">{subSchema.description}</p>
              ),
            ]
          : renderer(subSchema, capitalize(prop), level)

      return [
        ...form,
        ...(Array.isArray(formSection) ? formSection : [formSection]),
      ]
    },
    []
  )
}
