import { VNode } from '@stencil/core'
import { JSONSchema7Definition } from 'json-schema'

export type ConfigSchema = JSONSchema7Definition

export type FormElementOptions = {
  label?: string
  onChangeHandler?: (e: Event) => void
}

export type FormElement<S extends ConfigSchema> = (
  schema: S,
  options?: FormElementOptions
) => VNode | VNode[]
