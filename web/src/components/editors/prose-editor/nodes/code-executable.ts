import StencilaCodeExecutable from '../../../nodes/code-executable'
import { executableAttrs, StencilaExecutableView } from './executable'

export const codeExecutableAttrs = {
  ...executableAttrs,
  programmingLanguage: { default: '' },
  guessLanguage: { default: null },
  code: { default: '' },
}

export class StencilaCodeExecutableView<
  Type extends StencilaCodeExecutable
> extends StencilaExecutableView<Type> {
  /**
   * Override to update `CodeExecutable.outputs` after patches from server
   * are applied to the view's DOM
   */
  handleMutation(mutation: MutationRecord): void {
    super.handleMutation(mutation)

    const elem = mutation.target
    if (
      elem instanceof HTMLElement &&
      mutation.type === 'childList' &&
      elem.slot === 'outputs'
    ) {
      const transaction = this.view.state.tr.setNodeAttribute(
        this.getPos(),
        'outputs',
        elem.innerHTML
      )
      this.view.dispatch(transaction)
    }
  }
}
