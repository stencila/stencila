import { VNode } from '@stencil/core'
import { ConfigSchema } from '../formBuilder'

export type FormElementOptions = {
  label?: string
  onChangeHandler?: (e: Event) => void
}

export type FormElement<S extends ConfigSchema> = (
  schema: S,
  options?: FormElementOptions
) => VNode | VNode[]
