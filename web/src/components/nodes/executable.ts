import { html } from 'lit'
import { property, state } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, Mode } from '../../mode'

import StencilaEntity from './entity'

export type ExecuteStatus =
  | 'Scheduled'
  | 'ScheduledPreviouslyFailed'
  | 'Running'
  | 'RunningPreviouslyFailed'
  | 'Succeeded'
  | 'Failed'
  | 'Cancelled'

export type ExecuteRequired =
  | 'NeverExecuted'
  | 'SemanticsChanged'
  | 'DependenciesChanged'
  | 'DependenciesFailed'
  | 'Failed'
  | 'No'

/**
 * A base component to represent the `Executable` node type
 */
export default class StencilaExecutable extends StencilaEntity {
  @property({
    attribute: 'execute-status',
  })
  executeStatus?: ExecuteStatus

  @property({
    attribute: 'execute-required',
  })
  executeRequired?: ExecuteRequired

  @property({
    attribute: 'execute-count',
  })
  executeCount?: number

  /**
   * Is the node executable in the current mode
   */
  protected isExecutable(): boolean {
    const mode = currentMode()
    return mode >= Mode.Alter && mode != Mode.Edit
  }

  protected execute() {
    this.emit('stencila-document-execute', {
      nodeId: this.id,
      ordering: 'Topological',
    })
  }

  renderExecuteIcon(tw: TW) {
    const { icon, color, title } = this.runButtonFromStatusAndRequired(
      this.executeStatus,
      this.executeRequired
    )
    return this.isExecutable()
      ? html`<sl-tooltip content="${title}">
          <stencila-icon
            name="${icon}"
            @click="${this.execute}"
            class=${tw`cursor-pointer`}
          ></stencila-icon>
        </sl-tooltip>`
      : html`<sl-tooltip content="${title}">
          <stencila-icon name="${icon}"></stencila-icon>
        </sl-tooltip>`
  }

  runButtonFromStatusAndRequired(
    executeStatus?: ExecuteStatus,
    executeRequired?: ExecuteRequired
  ): {
    icon: string
    color: string
    title: string
  } {
    // If scheduled or running then show that status
    switch (executeStatus) {
      case 'Scheduled': {
        return {
          icon: 'timer',
          color: 'neutral-500, #6e7591',
          title: 'Scheduled',
        }
      }
      case 'ScheduledPreviouslyFailed': {
        return {
          icon: 'timer',
          color: 'danger-500, #cf445e',
          title: 'Scheduled (previously failed)',
        }
      }
      case 'Running': {
        return {
          icon: 'arrow-repeat',
          color: 'neutral-500, #6e7591',
          title: 'Running',
        }
      }
      case 'RunningPreviouslyFailed': {
        return {
          icon: 'arrow-repeat',
          color: 'danger-500, #cf445e',
          title: 'Running (previously failed)',
        }
      }
    }

    // Otherwise, if execution is required show why
    switch (executeRequired) {
      case 'NeverExecuted': {
        return {
          icon: 'dash-circle',
          color: 'neutral-500, #6e7591',
          title: 'Not run yet',
        }
      }
      case 'DependenciesFailed': {
        return {
          icon: 'refresh',
          color: 'danger-500, #cf445e',
          title: 'Dependencies failed',
        }
      }
      case 'DependenciesChanged': {
        return {
          icon: 'refresh',
          color: 'warn-600, #ba8925',
          title: 'Dependencies changed',
        }
      }
      case 'SemanticsChanged': {
        return {
          icon: 'refresh',
          color: 'warn-600, #ba8925',
          title: 'Semantics changed',
        }
      }
      case 'Failed': {
        return {
          icon: 'exclamation-circle',
          color: 'danger-500, #cf445e',
          title: 'Failed',
        }
      }
    }

    // Otherwise, show last status
    switch (executeStatus) {
      case 'Succeeded': {
        return {
          icon: 'check-circle',
          color: 'success-500, #3c8455',
          title: 'Succeeded',
        }
      }
      case 'Failed': {
        return {
          icon: 'exclamation-circle',
          color: 'danger-500, #cf445e',
          title: 'Failed',
        }
      }
      case 'Cancelled': {
        return {
          icon: 'slash-circle',
          color: 'warn-600, #ba8925',
          title: 'Cancelled',
        }
      }
    }

    // Although this should be redundant, it avoids this function every returning undefined
    // which causes other errors (e.g. if there is a patching error or a new variant added to
    // the above enums)
    return {
      icon: 'question',
      color: 'neutral-500, #6e7591',
      title: 'Unknown status',
    }
  }
}
