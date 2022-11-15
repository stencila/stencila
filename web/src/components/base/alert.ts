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
 * Alerts that are currently active
 */
const activeAlerts: Record<string, number> = {}

/**
 * Dynamically generate and pop up an alert
 *
 * Will not show the same notification if there is already one
 * with the same message. This avoids having a whole screen's
 * worth of notifications.
 */
export function notify(
  message: string,
  variant = 'primary',
  icon: IconName = 'info-circle',
  duration = 3000
) {
  if (activeAlerts[message] && activeAlerts[message] - Date.now() < duration) {
    return
  }

  const alert = Object.assign(document.createElement('stencila-alert'), {
    variant,
    closable: true,
    duration,
    innerHTML: `
        <stencila-icon name="${icon}" slot="icon"></stencila-icon>
        ${escapeHtml(message)}
      `,
  })

  activeAlerts[message] = Date.now()
  alert.addEventListener('sl-hide', () => delete activeAlerts[message])

  document.body.append(alert)

  // @ts-expect-error because `toast` is not defined on `StencilaAlert`
  return alert.toast()
}
