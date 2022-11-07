import StencilaStyled from '../../../nodes/styled'
import {
  codeExecutableAttrs,
  StencilaCodeExecutableView,
} from './code-executable'

export const styledAttrs = {
  ...codeExecutableAttrs,
  css: { default: '' },
}

export class StencilaStyledView<
  Type extends StencilaStyled
> extends StencilaCodeExecutableView<Type> {
  /**
   * Override to update `Styled.css` after patches from server
   * are applied to the view's DOM
   */
  handleMutation(mutation: MutationRecord): void {
    super.handleMutation(mutation)

    const elem = mutation.target
    if (
      elem instanceof HTMLElement &&
      mutation.type === 'childList' &&
      elem.slot === 'css'
    ) {
      const transaction = this.view.state.tr.setNodeAttribute(
        this.getPos(),
        'css',
        elem.innerHTML
      )
      this.view.dispatch(transaction)
    }
  }
}
