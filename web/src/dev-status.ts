import { html } from 'lit'

import './components/base/tag'

export enum DevStatus {
  Planned = 0,
  InProgress = 1,
  ComingSoon = 2,
  Alpha = 3,
  Beta = 4,
  Stable = 5,
}

/**
 * Get the label for a status
 */
export function devStatusLabel(status: DevStatus): string {
  switch (status) {
    case DevStatus.Planned:
      return 'Planned'
    case DevStatus.InProgress:
      return 'In progress'
    case DevStatus.ComingSoon:
      return 'Coming soon'
    case DevStatus.Alpha:
      return 'Alpha'
    case DevStatus.Beta:
      return 'Beta'
    case DevStatus.Stable:
      return 'Stable'
  }
}

/**
 * Get the "tag" element for a status
 */
export function devStatusTag(status: DevStatus) {
  switch (status) {
    case DevStatus.Planned:
      return html`<stencila-tag color="indigo">Planned</stencila-tag>`
    case DevStatus.InProgress:
      return html`<stencila-tag color="purple">In progress</stencila-tag>`
    case DevStatus.ComingSoon:
      return html`<stencila-tag color="blue">Coming soon</stencila-tag>`
    case DevStatus.Alpha:
      return html`<stencila-tag color="red">Alpha</stencila-tag>`
    case DevStatus.Beta:
      return html`<stencila-tag color="yellow">Beta</stencila-tag>`
    case DevStatus.Stable:
      return html`<stencila-tag color="green">Stable</stencila-tag>`
  }
}
