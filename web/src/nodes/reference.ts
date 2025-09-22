import { Author, CreativeWorkType, Reference as ReferenceType, Text, Inline, Person, PersonOrOrganization, IntegerOrString, StringOrNumber, PropertyValueOrString } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Reference` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/reference.md
 */
@customElement('stencila-reference')
@withTwind()
export class Reference extends Entity {
  @property({ attribute: 'appearance-index', type: Number })
  appearanceIndex?: number

  @property({ attribute: 'work-type' })
  workType?: CreativeWorkType

  @property()
  doi?: string

  @property({ type: Array })
  authors?: Author[]

  @property()
  date?: string

  @property({ attribute: 'is-part-of', type: Object })
  isPartOf?: ReferenceType

  @property({ attribute: 'page-start' })
  pageStart?: string

  @property({ attribute: 'page-end' })
  pageEnd?: string

  @property()
  pagination?: string

  @property()
  text?: string

  @property({ type: Array })
  editors?: Person[]

  @property({ type: Object })
  publisher?: PersonOrOrganization

  @property({ attribute: 'volume-number' })
  volumeNumber?: IntegerOrString

  @property({ attribute: 'issue-number' })
  issueNumber?: IntegerOrString

  @property()
  version?: StringOrNumber

  @property({ type: Array })
  identifiers?: PropertyValueOrString[]

  @property()
  url?: string

  override render() {
    return html`<slot name="content"></slot>`
  }

  renderWithinTooltip() {
    const index = this.appearanceIndex ? html`${this.appearanceIndex}. ` : ''

    if (this.text) {
      return html`<div class="font-sans text-xs">${index}${this.text}</div>`
    }

    // Links do not work within a <sl-tooltip>, nor does copy and pasting, so
    // this does not include the DOI

    const authors = this.authors
      ? this.authors.map(authorNameInitialsDotted).join(', ')
      : 'Anon'

    return html`<div class="font-sans text-xs">
      ${index}${authors}${this.date ? html` (${dateYear(this.date)}). ` : ''}<span
        class="font-semibold"
        ><slot name="title"></slot></span
      >.
      ${this.isPartOf
        ? html`<span class="italic"> ${partOf(this.isPartOf)}</span>`
        : ''}
    </div>`
  }

  renderDefault() {
    if (this.text) {
      return html`<div class="font-sans text-xs">${this.text}</div>`
    }

    const authors = this.authors
      ? this.authors.map(authorNameInitialsDotted).join(', ')
      : ''

    const year = this.date ? html` (${dateYear(this.date)}). ` : ''

    const title = html`<span class="font-semibold"
      ><slot name="title"></slot
    ></span>`

    const isPartOf = this.isPartOf
      ? html`<span class="italic"> ${partOf(this.isPartOf)}</span>`
      : ''

    const link = getLink(this)
    return html`<div class="font-sans text-xs">
      ${authors}${year}${title}${isPartOf}${link}
    </div>`
  }
}

/*
 * Extract a single surname for citation display purposes.
 * Prioritizes family names for persons, falls back to organization names.
 * Used in parenthetical citations where space is limited.
 */
function authorSingleName(author: Author): string {
  switch (author.type) {
    case 'Person':
      return author.familyNames?.[0] ?? author.name ?? 'Anon'
    case 'Organization':
    case 'SoftwareApplication':
      return author.name ?? author.alternateNames?.[0] ?? 'Anon'
    case 'AuthorRole':
      switch (author.author.type) {
        case 'Person':
        case 'Organization':
        case 'SoftwareApplication':
          return authorSingleName(author.author)
        case 'Thing':
          return (
            author.author.name ?? author.author.alternateNames?.[0] ?? 'Anon'
          )
      }
  }
}

/*
 * Format author name with dotted initials for full references.
 * Converts "John Smith" to "Smith, J." following academic conventions.
 * Falls back to organization names or single names when structured names unavailable.
 */
function authorNameInitialsDotted(author: Author): string {
  switch (author.type) {
    case 'Person':
      if (author.familyNames?.length && author.givenNames?.length > 0) {
        return (
          author.familyNames.filter((name) => name.length > 0).join(' ') +
          ', ' +
          author.givenNames
            .filter((name) => name.length > 0)
            .map((name) => `${name[0]}.`)
            .join('')
        )
      } else {
        return authorSingleName(author)
      }
    case 'Organization':
    case 'SoftwareApplication':
      return authorSingleName(author)
    case 'AuthorRole':
      switch (author.author.type) {
        case 'Person':
          return authorNameInitialsDotted(author.author)
        case 'Organization':
        case 'SoftwareApplication':
        case 'Thing':
          return authorSingleName(author)
      }
  }
}

/*
 * Extract four-digit year from ISO date string.
 * Simple string slicing assuming standard ISO format (YYYY-MM-DD).
 */
function dateYear(date?: string): string {
  return date?.slice(0, 4) ?? ''
}

/*
 * Format journal or publication information from isPartOf reference.
 * Combines title with volume and issue numbers in parentheses.
 * Used for displaying journal information in legacy rendering methods.
 */
function partOf(ref: ReferenceType): string {
  // Build the title with volume/issue info
  let result = ''

  // Get the title from the reference
  if (ref.title) {
    function inlineToString(inline: Inline) {
      if (typeof inline === 'string') return inline
      if (typeof inline === 'number') return inline.toString()
      if (typeof inline === 'boolean') return inline.toString()
      if (inline && typeof inline === 'object' && 'type' in inline) {
        if (inline.type === 'Text') {
          return (inline as Text).value.string || ''
        } else if ('content' in inline) {
          (inline.content as Inline[]).map((inline) => inlineToString(inline)).join('')
        }
      }
      return ''
    }
    result = ref.title.map(inlineToString).join('')
  }

  // Add volume number if present
  if (ref.volumeNumber) {
    result += ` ${ref.volumeNumber}`
  }

  // Add issue number in parentheses if present
  if (ref.issueNumber) {
    result += ` (${ref.issueNumber})`
  }

  return result
}

/*
 * Generate appropriate Link for a reference based on available identifiers.

 * Prioritizes DOI links over other URLs, filters out placeholder DOIs.
 * Returns null if no valid URL can be constructed.
 */
function getLink(reference: Reference) {
  let url

  if (reference.doi && !reference.doi.startsWith('10.0000')) {
    url = `https://doi.org/${reference.doi}`
  }

  if (reference.url) {
    url = reference.url
  }

  if (reference.isPartOf?.url) {
    url = reference.isPartOf.url
  }

  return url
    ? html`. <a href="${url}" target="_blank">${url}</a>`
    : ''
}
