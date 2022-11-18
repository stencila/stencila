import StencilaCodeStatic from '../../../nodes/code-static'
import { entityAttrs, StencilaEntityView } from './entity'

export const codeStaticAttrs = {
  ...entityAttrs,
  programmingLanguage: { default: 'unknown' },
  code: { default: '' },
}

export class StencilaCodeStaticView<
  Type extends StencilaCodeStatic
> extends StencilaEntityView<Type> {}
