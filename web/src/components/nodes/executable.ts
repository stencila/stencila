import { html } from 'lit'
import { property } from 'lit/decorators'
import { TW } from 'twind'
import 'twind/colors'
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
    const { title, icon, color } = this.executeIconFromStatusAndRequired(
      this.executeStatus,
      this.executeRequired
    )
    const isExecutable = this.isExecutable()
    return html`<sl-tooltip content="${title}">
      <stencila-icon
        name=${icon}
        @click=${isExecutable ? this.execute : null}
        class=${isExecutable
          ? tw`text-${color}-600 ${
              this.executeStatus === 'Running'
                ? 'cursor-wait'
                : 'cursor-pointer'
            }`
          : tw`text-${color}-600`}
      ></stencila-icon>
    </sl-tooltip>`
  }

  executeIconFromStatusAndRequired(
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
          icon: 'stopwatch',
          color: 'green',
          title: 'Scheduled',
        }
      }
      case 'ScheduledPreviouslyFailed': {
        return {
          icon: 'stopwatch',
          color: 'red',
          title: 'Scheduled (previously failed)',
        }
      }
      case 'Running': {
        return {
          icon: 'arrow-repeat',
          color: 'green',
          title: 'Running',
        }
      }
      case 'RunningPreviouslyFailed': {
        return {
          icon: 'arrow-repeat',
          color: 'red',
          title: 'Running (previously failed)',
        }
      }
    }

    // Otherwise, if execution is required show why
    switch (executeRequired) {
      case 'NeverExecuted': {
        return {
          icon: 'dash-circle',
          color: 'neutral',
          title: 'Not run yet',
        }
      }
      case 'DependenciesFailed': {
        return {
          icon: 'arrow-clockwise',
          color: 'red',
          title: 'Dependencies failed',
        }
      }
      case 'DependenciesChanged': {
        return {
          icon: 'arrow-clockwise',
          color: 'amber',
          title: 'Dependencies changed',
        }
      }
      case 'SemanticsChanged': {
        return {
          icon: 'arrow-clockwise',
          color: 'amber',
          title: 'Semantics changed',
        }
      }
      case 'Failed': {
        return {
          icon: 'exclamation-circle',
          color: 'red',
          title: 'Failed',
        }
      }
    }

    // Otherwise, show last status
    switch (executeStatus) {
      case 'Succeeded': {
        return {
          icon: 'check-circle',
          color: 'green',
          title: 'Succeeded',
        }
      }
      case 'Failed': {
        return {
          icon: 'exclamation-circle',
          color: 'red',
          title: 'Failed',
        }
      }
      case 'Cancelled': {
        return {
          icon: 'slash-circle',
          color: 'amber',
          title: 'Cancelled',
        }
      }
    }

    // Although this should be redundant, it avoids this function every returning undefined
    // which causes other errors (e.g. if there is a patching error or a new variant added to
    // the above enums)
    return {
      icon: 'question',
      color: 'neutral',
      title: 'Unknown status',
    }
  }
}
