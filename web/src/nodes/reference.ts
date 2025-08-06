import { Author, CreativeWorkType, Reference as ReferenceType, Text } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Citation } from './citation'
import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Reference` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/reference.md
 */
@customElement('stencila-reference')
@withTwind()
export class Reference extends Entity {
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

  @property({ attribute: 'pagination' })
  pagination?: string

  override render() {
    const cite = this.closestGlobally('stencila-citation') as Citation | null
    if (cite) {
      return this.renderWithinCitation(cite)
    }

    const article = this.closestGlobally('stencila-article > [slot=references]')
    if (article) {
      return this.renderWithinReferences()
    }

    return this.renderDefault()
  }

  renderWithinCitation(cite: Citation) {
    let author = this.authors?.[0] ? authorSingleName(this.authors[0]) : 'Anon'
    if (this.authors?.length == 2) {
      const second = this.authors[1]
        ? authorSingleName(this.authors[1])
        : 'Anon'
      author += ' & ' + second
    } else if (this.authors?.length > 2) {
      author += ' et al.'
    }

    const year = dateYear(this.date)

    const repr = (() => {
      switch (cite.citationMode) {
        case 'Narrative':
          return html`${author} (${year})`
        case 'NarrativeAuthor':
          return html`${author}`
        case 'Parenthetical':
          return html`${author}, ${year}`
        default:
          return html`${author} ${year}`
      }
    })()

    return html`<sl-tooltip class="inline-block"
      ><span>${repr}</span
      ><span slot="content">${this.renderWithinTooltip()}</span></sl-tooltip
    >`
  }

  renderWithinTooltip() {
    // Links do not work within a <sl-tooltip>, nor does copy and pasting, so
    // this does not include the DOI

    const authors = this.authors
      ? this.authors.map(authorNameInitialsDotted).join(', ')
      : 'Anon'

    return html`<div class="font-sans text-xs">
      ${authors}${this.date ? html` (${dateYear(this.date)}). ` : ''}<span
        class="font-semibold"
        ><slot name="title"></slot></span
      >.
      ${this.isPartOf
        ? html`<span class="italic"> ${partOf(this.isPartOf)}</span>`
        : ''}
    </div>`
  }

  renderWithinReferences() {
    const authors = this.authors
      ? this.authors.map(authorNameInitialsDotted).join(', ')
      : 'Anon'

    const year = this.date ? html` (${dateYear(this.date)}). ` : ''

    const isPartOf = this.isPartOf ? html`<em> ${partOf(this.isPartOf)}</em>` : ''

    const pagination = pagesEndashed(this.pageStart, this.pageStart, this.pagination)
    const pages = pagination
        ? html` ${pagination}`
        : ''

    const url = createUrl(this)
    const link = url
        ? html` <a href="url" target="_blank">url</a>`
        : ''

    return html`<div class="mt-3">
      ${authors}${year}<slot name="title"></slot>.${isPartOf}${pages}${link}
    </div>`
  }

  renderDefault() {
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

    const url = createUrl(this)
    const link = url
      ? html` <a href="${url}" target="_blank"
          ><stencila-ui-icon
            class="inline-block"
            name="externalLink"
          ></stencila-ui-icon
        ></a>`
      : ''

    return html`<div class="font-sans text-xs">
      ${authors}${year}${title}${isPartOf}${link}
    </div>`
  }
}

/**
 * Get a single name for an author
 *
 * Used for representing an author within a `Citation` e.g.
 * (Smith & Jones, 1990)
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

/**
 * Get the name and initials (with dots) for an author
 *
 * Used when representing an author within a full reference e.g.
 * Smith, J. & Jones, T. (1990)
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

/**
 * Get the year of a date
 */
function dateYear(date: string): string {
  return date.slice(0, 4)
}

/**
 * Render the `isPartOf` property as a string
 */
function partOf(ref: ReferenceType): string {
  // Build the title with volume/issue info
  let result = ''
  
  // Get the title from the reference
  if (ref.title) {
    result = ref.title.map(inline => {
      if (typeof inline === 'string') return inline
      if (typeof inline === 'number') return inline.toString()
      if (typeof inline === 'boolean') return inline.toString()
      if (inline && typeof inline === 'object' && 'type' in inline) {
        if (inline.type === 'Text') {
          return (inline as Text).value || ''
        }
      }
      return ''
    }).join('')
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

/**
 * Render the pagination properties
 */
function pagesEndashed(
  pageStart?: string,
  pageEnd?: string,
  pagination?: string
): string {
  return pageStart && pageStart.length > 0
    ? pageEnd && pageEnd.length > 0
      ? `${pageStart}–${pageEnd}`
      : pageStart
    : pagination && pagination.length > 0
      ? pagination
      : ''
}

/**
 * Create a URL for a reference
 */
function createUrl(reference: Reference): string | null {
  if (reference.doi && !reference.doi.startsWith('10.0000')) {
    return `https://doi.org/${reference.doi}`
  }

  if (reference.isPartOf?.url) {
    return reference.isPartOf.url
  }

  return null
}
