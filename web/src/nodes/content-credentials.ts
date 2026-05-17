/**
 * Utilities for rendering advisory Content Credentials UI for media assets.
 *
 * Rendered Stencila media elements can carry a `content-credentials-c2pa`
 * summary attribute. This is a display hint only; verification happens against
 * the signed asset bytes through verify.stencila.io.
 */
import { type TemplateResult, html } from 'lit'
import { unsafeSVG } from 'lit/directives/unsafe-svg.js'
import moment from 'moment'

import contentCredentialsIcon from '../ui/icons/content-credentials.svg?raw'
import { getModeParam } from '../utilities/getModeParam'

interface C2paAction {
  action?: string
  description?: string
  when?: string
}

interface C2paIngredient {
  title?: string
  relationship?: string
  format?: string
  activeManifest?: string
}

interface C2paSummary {
  issuer?: string
  issuedAt?: string
  device?: string
  generator?: string
  actions?: C2paAction[]
  ingredients?: C2paIngredient[]
}

const LUCIDE_ICONS = {
  activity:
    '<path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"/>',
  boxes:
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M2.97 12.92A2 2 0 0 0 2 14.63v3.24a2 2 0 0 0 .97 1.71l3 1.8a2 2 0 0 0 2.06 0L12 19v-5.5l-5-3zM7 16.5l-4.74-2.85M7 16.5l5-3m-5 3v5.17m5-8.17V19l3.97 2.38a2 2 0 0 0 2.06 0l3-1.8a2 2 0 0 0 .97-1.71v-3.24a2 2 0 0 0-.97-1.71L17 10.5zm5 3l-5-3m5 3l4.74-2.85M17 16.5v5.17"/><path d="M7.97 4.42A2 2 0 0 0 7 6.13v4.37l5 3l5-3V6.13a2 2 0 0 0-.97-1.71l-3-1.8a2 2 0 0 0-2.06 0zM12 8L7.26 5.15M12 8l4.74-2.85M12 13.5V8"/></g>',
  'external-link':
    '<path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 3h6v6m-11 5L21 3m-3 10v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
  'file-input':
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M4 11V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.706.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-1"/><path d="M14 2v5a1 1 0 0 0 1 1h5M2 15h10m-3 3l3-3l-3-3"/></g>',
  'file-output':
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M4.226 20.925A2 2 0 0 0 6 22h12a2 2 0 0 0 2-2V8a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v3.127"/><path d="M14 2v5a1 1 0 0 0 1 1h5M5 11l-3 3m3 3l-3-3h10"/></g>',
  'folder-open':
    '<path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m6 14l1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>',
  layers:
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83z"/><path d="M2 12a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 12"/><path d="M2 17a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 17"/></g>',
  network:
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><rect width="6" height="6" x="16" y="16" rx="1"/><rect width="6" height="6" x="2" y="16" rx="1"/><rect width="6" height="6" x="9" y="2" rx="1"/><path d="M5 16v-3a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3m-7-4V8"/></g>',
  package:
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73zm1 .27V12"/><path d="M3.29 7L12 12l8.71-5M7.5 4.27l9 5.15"/></g>',
  play:
    '<path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"/>',
  sparkles:
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594zM20 2v4m2-2h-4"/><circle cx="4" cy="20" r="2"/></g>',
} as const

type LucideIcon = keyof typeof LUCIDE_ICONS

export interface ContentCredentialAsset {
  contentUrl: string
  c2pa?: C2paSummary
}

/**
 * Read advisory Content Credentials metadata from a media custom element.
 */
export function readContentCredentialAsset(
  element: HTMLElement,
  contentUrl: string | undefined
): ContentCredentialAsset | undefined {
  if (!contentUrl || !element.hasAttribute('content-credentials')) {
    return undefined
  }

  return {
    contentUrl,
    c2pa: jsonAttr<C2paSummary>(element, 'content-credentials-c2pa'),
  }
}

/**
 * Build a verify.stencila.io URL for signed asset bytes.
 */
export function buildContentCredentialVerifyUrl(
  asset: ContentCredentialAsset,
  documentUrl = currentDocumentUrl()
): string | undefined {
  const sourceUrl = absoluteHttpUrl(asset.contentUrl, documentUrl)
  if (!sourceUrl) {
    return undefined
  }

  const params = new URLSearchParams({ source: sourceUrl })

  return `https://verify.stencila.io?${params.toString()}`
}

/**
 * Render the Content Credentials pin and compact metadata card for an asset.
 */
export function renderContentCredentialPin(
  asset: ContentCredentialAsset | undefined,
  cardOpen: boolean,
  onOpen: () => void,
  onClose: () => void,
  onToggle: () => void
): TemplateResult | string {
  if (!asset) {
    return ''
  }

  const verifyUrl = buildContentCredentialVerifyUrl(asset)
  const actions = asset.c2pa?.actions?.filter((action) => action.action) ?? []
  const ingredients = asset.c2pa?.ingredients ?? []
  const detailRows = [
    ['Issuer', asset.c2pa?.issuer],
    ['Issued', formatTimeAgo(asset.c2pa?.issuedAt)],
    ['Device', asset.c2pa?.device],
  ].filter((row): row is [string, string] => Boolean(row[1]))

  const onClick = (event: MouseEvent) => {
    event.stopPropagation()

    if (window.matchMedia('(hover: hover)').matches) {
      onOpen()
    } else {
      onToggle()
    }
  }
  const onFocusOut = (event: FocusEvent) => {
    const current = event.currentTarget as HTMLElement

    if (event.relatedTarget instanceof Node && current.contains(event.relatedTarget)) {
      return
    }

    onClose()
  }

  return html`
    <div
      class="content-credentials"
      @mouseenter=${onOpen}
      @mouseleave=${onClose}
      @focusin=${onOpen}
      @focusout=${onFocusOut}
    >
      <button
        class="content-credentials-pin"
        aria-label="Content Credentials"
        aria-expanded=${cardOpen ? 'true' : 'false'}
        @click=${onClick}
        type="button"
      >
        ${unsafeSVG(contentCredentialsIcon)}
      </button>
      ${cardOpen
        ? html`
            <div
              class="content-credentials-card"
              role="dialog"
              aria-label="Content Credentials"
            >
              <div class="content-credentials-note">
                This image has embedded 
                <a
                  href="https://contentcredentials.org/"
                  target="_blank"
                  rel="noopener"
                >
                  Content Credentials
                </a>.
              </div>
              ${detailRows.length
                ? html`
                    <dl class="content-credentials-details">
                      ${detailRows.map(
                        ([label, value]) => html`
                          <div>
                            <dt>${label}</dt>
                            <dd>${value}</dd>
                          </div>
                        `
                      )}
                    </dl>
                  `
                : ''}
              ${actions.length
                ? html`
                    <div class="content-credentials-section">
                      <div class="content-credentials-section-title">Actions</div>
                      <ul class="content-credentials-list">
                        ${actions.map(
                          (action) => html`
                            <li>
                              ${renderLucideIcon(actionIcon(action.action))}
                              <span class="content-credentials-list-content">
                                <span class="content-credentials-list-main">
                                  ${action.description ?? actionLabel(action.action)}
                                </span>
                                ${action.when
                                  ? html`<span class="content-credentials-list-meta">
                                      ${formatTimeAgo(action.when)}
                                    </span>`
                                  : ''}
                              </span>
                            </li>
                          `
                        )}
                      </ul>
                    </div>
                  `
                : ''}
              ${ingredients.length
                ? html`
                    <div class="content-credentials-section">
                      <div class="content-credentials-section-title">Ingredients</div>
                      <ul class="content-credentials-list">
                        ${ingredients.map(
                          (ingredient) => html`
                            <li>
                              ${renderLucideIcon(ingredientIcon(ingredient.relationship))}
                              <span class="content-credentials-list-content">
                                <span class="content-credentials-badge">
                                  ${relationshipLabel(ingredient.relationship)}
                                </span>
                                <span class="content-credentials-list-main">
                                  ${ingredient.title ?? ingredient.format ?? 'Ingredient'}
                                </span>
                                ${ingredient.format && ingredient.format !== ingredient.title
                                  ? html`<span class="content-credentials-list-meta">
                                      ${ingredient.format}
                                    </span>`
                                  : ''}
                              </span>
                            </li>
                          `
                        )}
                      </ul>
                    </div>
                  `
                : ''}
              ${verifyUrl
                ? html`
                    <div class="content-credentials-verify-row">
                      <a
                        class="content-credentials-verify"
                        href=${verifyUrl}
                        target="_blank"
                        rel="noopener"
                      >
                        Verify
                        ${renderLucideIcon('external-link', 'content-credentials-verify-icon')}
                      </a>
                    </div>
                  `
                : ''}
            </div>
          `
        : ''}
    </div>
  `
}

function attr(element: HTMLElement, name: string): string | undefined {
  return element.getAttribute(name) || undefined
}

function jsonAttr<T>(element: HTMLElement, name: string): T | undefined {
  const value = attr(element, name)
  if (!value) {
    return undefined
  }

  try {
    return JSON.parse(value) as T
  } catch {
    return undefined
  }
}

function renderLucideIcon(
  icon: LucideIcon,
  className = 'content-credentials-list-icon'
): TemplateResult {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${LUCIDE_ICONS[icon]}</svg>`

  return html`
    <span
      class=${`${className} i-lucide:${icon}`}
      aria-hidden="true"
    >
      ${unsafeSVG(svg)}
    </span>
  `
}

function actionIcon(action: string | undefined): LucideIcon {
  switch (action) {
    case 'c2pa.created':
      return 'sparkles'
    case 'c2pa.opened':
      return 'folder-open'
    case 'c2pa.placed':
      return 'layers'
    case 'c2pa.exported':
      return 'file-output'
    case 'org.stencila.executed':
      return 'play'
    default:
      return 'activity'
  }
}

function actionLabel(action: string | undefined): string {
  switch (action) {
    case 'c2pa.created':
      return 'Created'
    case 'c2pa.opened':
      return 'Opened'
    case 'c2pa.placed':
      return 'Placed'
    case 'c2pa.exported':
      return 'Exported'
    case 'org.stencila.executed':
      return 'Executed'
    default:
      return humanizeToken(action ?? 'Action')
  }
}

function ingredientIcon(relationship: string | undefined): LucideIcon {
  switch (relationship) {
    case 'inputTo':
      return 'file-input'
    case 'componentOf':
      return 'boxes'
    case 'parentOf':
      return 'network'
    default:
      return 'package'
  }
}

function relationshipLabel(relationship: string | undefined): string {
  switch (relationship) {
    case 'inputTo':
      return 'Input'
    case 'componentOf':
      return 'Component'
    case 'parentOf':
      return 'Parent'
    default:
      return humanizeToken(relationship ?? 'Ingredient')
  }
}

function humanizeToken(value: string): string {
  const token = value.split('.').pop() ?? value
  return token
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/[-_]/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
}

function formatTimeAgo(value: string | undefined): string | undefined {
  if (!value) {
    return undefined
  }

  if (
    typeof window !== 'undefined' &&
    getModeParam(window) === 'test-expand-all'
  ) {
    return 'some time ago'
  }

  const date = moment(value)
  if (!date.isValid()) {
    return value
  }

  return date.fromNow()
}

/**
 * Resolve a URL when it can be represented as an absolute HTTP URL.
 */
function absoluteHttpUrl(url: string, documentUrl: string): string | undefined {
  try {
    const resolved = new URL(url, documentUrl)
    if (resolved.protocol === 'http:' || resolved.protocol === 'https:') {
      return resolved.href
    }
  } catch {
    return undefined
  }

  return undefined
}

function currentDocumentUrl(): string {
  return typeof window === 'undefined' ? 'file:///' : window.location.href
}
