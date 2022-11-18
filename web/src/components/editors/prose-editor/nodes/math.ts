import StencilaMath from '../../../nodes/math'
import { executableAttrs, StencilaExecutableView } from './executable'

export const mathAttrs = {
  ...executableAttrs,
  mathLanguage: { default: 'tex' },
  code: { default: '' },
  mathml: { default: '' },
}

export class StencilaMathView<
  Type extends StencilaMath
> extends StencilaExecutableView<Type> {}
