import '@shoelace-style/shoelace/dist/components/icon/icon'
import { AuthorRoleName } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'

import '../last-modified'

/**
 * A component for displaying an `Author` within the `authors` property of nodes
 *
 * In the Stencila Schema [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)
 * is a "Union type for things that can be an author of a CreativeWork or other type":
 *
 * - Person
 * - Organization
 * - SoftwareApplication
 * - AuthorRole
 *
 * Note: the above is NOT a list of the properties of an author, it is a list of the types that can
 * be an author. Do not let this confuse you :)
 *
 * `Author` is implemented as a UI component here so that we have a uniform way of displaying all the different
 * types which can be considered an author and which can thus be in the `authors` property. This component
 * exposes properties that the various types can bind to. Properties are used rather than slots to maintain
 * typing and because they are all simple atomic values.
 */
@customElement('stencila-ui-node-author')
@withTwind()
export class UINodeAuthor extends LitElement {
  /**
   * The type of node that is the author
   *
   * Might be useful for determining default icons etc.
   */
  @property()
  type: 'Person' | 'Organization' | 'SoftwareApplication'

  /**
   * The id of the author
   *
   * Currently only available for `SoftwareApplication` authors. The intension is
   * to use this to be able to fetch more information about the authors (e.g. affiliation)
   * from some canonical source in the document. But currently this has not been finalized
   * (or even started).
   */
  @property({ attribute: '_id' })
  $id?: string

  /**
   * The name of the author
   *
   * Should be available for all author types, for a `Person` by concatenating
   *  `givenNames` and `familyNames`.
   */
  @property()
  name: string

  /**
   * The role that the author has
   *
   * Only available for `AuthorRole` authors.
   */
  @property({ attribute: 'role' })
  roleName?: AuthorRoleName

  /**
   * The format the the author used
   *
   * Only available for `AuthorRole` authors.
   */
  @property()
  format?: string

  /**
   * The timestamp of the last modification made by the author in a particular role
   *
   * Only available for `AuthorRole` authors.
   */
  @property({ type: Number })
  timestamp?: number

  /**
   * Additional details to display (e.g. a person's organization, the software version).
   */
  @property()
  details?: string

  override render() {
    const roleLabel = (() => {
      switch (this.roleName) {
        case 'Instructor':
          return 'Instructed'
        case 'Prompter':
          return 'Prompted'
        case 'Generator':
          return 'Generated'
        case 'Accepter':
          return 'Accepted'
        case 'Writer':
          return 'Wrote'
        default:
          return 'Contributed'
      }
    })()

    return html`<div class="@container w-full">
      <div class="flex flex-col gap-x-2 font-sans mb-4 @xs:flex-row @xs:mb-0">
        <div class="flex flex-row flex-grow">
          <div class="flex items-center justify-center mr-2">
            <div
              class="w-6 h-6 flex items-center justify-center grow-0 stretch-0"
            >
              ${this.renderIconOrAvatar()}
            </div>
          </div>
          <div class="grow flex flex-col justify-center">
            <span
              class=${`text-2xs leading-none ${this.roleName ? '' : 'hidden'}`}
              >${this.roleName}
              ${this.format
                ? html`<span class="opacity-60"> ${this.format}</span>`
                : ''}</span
            >
            <span
              class="text-xs leading-5 overflow-hidden whitespace-nowrap text-ellipsis inline-block"
              >${this.name}</span
            >
            <span
              class=${`text-2xs leading-none overflow-hidden whitespace-nowrap text-ellipsis inline-block ${this.details ? '' : 'hidden'}`}
              >${this.details}</span
            >
          </div>
        </div>
        <div
          class=${`grow-0 shrink-0 text-2xs @xs:text-right ${this.roleName ? 'pt-3' : 'pt-1'}`}
        >
          <stencila-ui-node-last-modified
            value=${this.timestamp}
            role-label=${roleLabel}
          ></stencila-ui-node-last-modified>
        </div>
      </div>
    </div>`
  }

  private renderSoftwareIcon() {
    const [provider] = this.$id?.trim().split('/') ?? []

    // Providers for which an icon is defined
    const iconName = ['anthropic', 'google', 'mistral', 'openai'].includes(
      provider
    )
      ? provider
      : undefined

    if (!iconName) {
      return this.renderAvatar()
    }

    return html`<sl-icon
      name=${iconName}
      library="stencila"
      class=${`text-2xl`}
    ></sl-icon>`
  }

  private renderOrgIcon() {
    return html`<sl-icon name="building" class=${`text-xl m-auto`}></sl-icon>`
  }

  private renderAvatar() {
    const classes = apply([
      'grid items-center justify-center',
      'w-6 h-6',
      'overflow-clip',
      'rounded-full',
      'bg-black/90',
    ])
    return html`<div class=${classes}>
      <span class="text-white text-xs leading-none m-auto mix-blend-difference"
        >${this.name.charAt(0)}</span
      >
    </div>`
  }

  private renderIconOrAvatar() {
    switch (this.type) {
      case 'SoftwareApplication':
        return this.renderSoftwareIcon()
      case 'Organization':
        return this.renderOrgIcon()
      default:
        return this.renderAvatar()
    }
  }
}
