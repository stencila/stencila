import StencilaExecutable from '../../../nodes/executable'
import { entityAttrs, StencilaEntityView } from './entity'

export const executableAttrs = {
  ...entityAttrs,
  errors: { default: '' },
}

export class StencilaExecutableView<
  Type extends StencilaExecutable
> extends StencilaEntityView<Type> {
  /**
   * Override to update `Executable.errors` after patches from server
   * are applied to the view's DOM
   */
  handleMutation(mutation: MutationRecord): void {
    super.handleMutation(mutation)

    const elem = mutation.target
    if (
      elem instanceof HTMLElement &&
      mutation.type === 'childList' &&
      elem.slot === 'errors'
    ) {
      const transaction = this.view.state.tr.setNodeAttribute(
        this.getPos(),
        'errors',
        elem.innerHTML
      )
      this.view.dispatch(transaction)
    }
  }
}
