import { customElement } from 'lit/decorators'

import { SlAlert } from '@shoelace-style/shoelace'

import { escapeHtml } from '../utils/html'
import { IconName } from './icon'

/**
 * An alert
 *
 * Currently just an alias for the Shoelace `SlAlert` component
 */
@customElement('stencila-alert')
export default class StencilaAlert extends SlAlert {}

/**
 * Dynamically generate and pop up an alert
 */
export function notify(
  message,
  variant = 'primary',
  icon: IconName = 'info-circle',
  duration = 3000
) {
  const alert = Object.assign(document.createElement('stencila-alert'), {
    variant,
    closable: true,
    duration: duration,
    innerHTML: `
        <stencila-icon name="${icon}" slot="icon"></stencila-icon>
        ${escapeHtml(message)}
      `,
  })

  document.body.append(alert)

  // @ts-expect-error because `toast` is not defined on `StencilaAlert`
  return alert.toast()
}
