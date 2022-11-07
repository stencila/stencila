import StencilaMath from '../../../nodes/math'
import { executableAttrs, StencilaExecutableView } from './executable'

export const mathAttrs = {
  ...executableAttrs,
  text: { default: '' },
  mathml: { default: '' },
}

export class StencilaMathView<
  Type extends StencilaMath
> extends StencilaExecutableView<Type> {}
