import { Author, CreativeWorkType, Reference as ReferenceType, Text, Inline, Person, PersonOrOrganization, IntegerOrString, StringOrNumber, PropertyValueOrString } from '@stencila/types'
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

  renderWithinReferences() {
    return this.renderAPAReference()
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

  /*
   * Render reference as author-year citation for in-text use.
   * Formats according to citation mode: narrative shows "Smith (2020)",
   * parenthetical shows "(Smith, 2020)", author-only shows just "Smith".
   */
  renderAuthorYearCitation(citationMode?: string) {
    let author = this.authors?.[0] ? authorSingleName(this.authors[0]) : 'Anon'
    if (this.authors?.length == 2) {
      const second = this.authors[1] ? authorSingleName(this.authors[1]) : 'Anon'
      author += ' & ' + second
    } else if (this.authors?.length > 2) {
      author += ' et al.'
    }

    const year = dateYear(this.date)

    switch (citationMode) {
      case 'Narrative':
        return html`${author} (${year})`
      case 'NarrativeAuthor':
        return html`${author}`
      case 'Parenthetical':
      default:
        return html`(${author}, ${year})`
    }
  }

  /*
   * Render reference as bracketed numeric citation like "[1]".
   * Uses the reference's appearance index to determine the number.
   */
  renderNumericBracketedCitation() {
    const index = this.appearanceIndex ?? 1
    return html`[${index}]`
  }

  /*
   * Render reference as parenthetical numeric citation like "(1)".
   * Uses the reference's appearance index to determine the number.
   */
  renderNumericParentheticalCitation() {
    const index = this.appearanceIndex ?? 1
    return html`(${index})`
  }

  /*
   * Render reference as superscript numeric citation like "¹".
   * Uses the reference's appearance index to determine the number.
   */
  renderNumericSuperscriptCitation() {
    const index = this.appearanceIndex ?? 1
    return html`<sup>${index}</sup>`
  }

  /*
   * Render full reference in APA (American Psychological Association) style.
   * Emphasizes publication year, uses sentence case for titles, and includes DOI.
   * Format: Author, A. A. (Year). Title of article. Journal Name, Volume(Issue), pages. DOI
   */
  renderAPAReference() {
    if (this.text) {
      return html`<div class="mt-3">${this.text}</div>`
    }

    const authors = formatAPAAuthors(this.authors)
    const year = this.date ? ` (${dateYear(this.date)}). ` : '. '
    const title = this.workType === 'Article' 
      ? html`<slot name="title"></slot>. `
      : html`<em><slot name="title"></slot></em>. `
    
    const journal = this.isPartOf ? html`<em>${formatJournalAPA(this.isPartOf)}</em>` : ''
    const volume = this.isPartOf?.volumeNumber ? html`, <em>${this.isPartOf.volumeNumber}</em>` : ''
    const issue = this.isPartOf?.issueNumber ? html`(${this.isPartOf.issueNumber})` : ''
    const pages = formatPagesAPA(this.pageStart, this.pageEnd, this.pagination)
    const link = getLink(this)

    return html`<div class="mt-3">
      ${authors}${year}${title}${journal}${volume}${issue}${pages}${link}.
    </div>`
  }

  /*
   * Render full reference in MLA (Modern Language Association) style.
   * Emphasizes author names, uses title case, and ends with period.
   * Format: Last, First. "Article Title." Journal Name, vol. #, no. #, Year, pp. ##-##.
   */
  renderMLAReference() {
    if (this.text) {
      return html`<div class="mt-3">${this.text}</div>`
    }

    const authors = formatMLAAuthors(this.authors)
    const title = this.workType === 'Article' 
      ? html`"<slot name="title"></slot>." `
      : html`<em><slot name="title"></slot></em>. `
    
    const journal = this.isPartOf ? html`<em>${getTitle(this.isPartOf)}</em>` : ''
    const volume = this.isPartOf?.volumeNumber ? html`, vol. ${this.isPartOf.volumeNumber}` : ''
    const issue = this.isPartOf?.issueNumber ? html`, no. ${this.isPartOf.issueNumber}` : ''
    const date = this.date ? html`, ${dateYear(this.date)}` : ''
    const pages = formatPagesMLA(this.pageStart, this.pageEnd, this.pagination)
    const link = getLink(this)

    return html`<div class="mt-3">
      ${authors}${title}${journal}${volume}${issue}${date}${pages}${link}.
    </div>`
  }

  /*
   * Render full reference in Chicago Manual of Style format.
   * Similar to MLA but with different punctuation and page formatting.
   * Format: Last, First. "Article Title." Journal Name Volume, no. Issue (Year): pages.
   */
  renderChicagoReference() {
    if (this.text) {
      return html`<div class="mt-3">${this.text}</div>`
    }

    const authors = formatChicagoAuthors(this.authors)
    const title = this.workType === 'Article' 
      ? html`"<slot name="title"></slot>." `
      : html`<em><slot name="title"></slot></em>. `
    
    const journal = this.isPartOf ? html`<em>${getTitle(this.isPartOf)}</em> ` : ''
    const volume = this.isPartOf?.volumeNumber ? html`${this.isPartOf.volumeNumber}` : ''
    const issue = this.isPartOf?.issueNumber ? html`, no. ${this.isPartOf.issueNumber}` : ''
    const date = this.date ? html` (${dateYear(this.date)})` : ''
    const pages = formatPagesChicago(this.pageStart, this.pageEnd, this.pagination)
    const link = getLink(this)

    return html`<div class="mt-3">
      ${authors}${title}${journal}${volume}${issue}${date}${pages}${link}.
    </div>`
  }

  /*
   * Render full reference in Vancouver/ICMJE style for medical journals.
   * Uses abbreviated author names and specific punctuation patterns.
   * Format: Last F. Article title. Journal Name. Year;Volume(Issue):pages.
   */
  renderVancouverReference() {
    if (this.text) {
      return html`<div class="mt-3">${this.appearanceIndex ?? 1}. ${this.text}</div>`
    }

    const index = this.appearanceIndex ?? 1
    const authors = formatVancouverAuthors(this.authors)
    const title = html`<slot name="title"></slot>. `
    const journal = this.isPartOf ? html`${getTitle(this.isPartOf)}. ` : ''
    const date = this.date ? html`${dateYear(this.date)}` : ''
    const volume = this.isPartOf?.volumeNumber ? html`;${this.isPartOf.volumeNumber}` : ''
    const issue = this.isPartOf?.issueNumber ? html`(${this.isPartOf.issueNumber})` : ''
    const pages = formatPagesVancouver(this.pageStart, this.pageEnd, this.pagination)
    const link = getLink(this)

    return html`<div class="mt-3">
      ${index}. ${authors}${title}${journal}${date}${volume}${issue}${pages}${link}.
    </div>`
  }

  /*
   * Render full reference in IEEE (Institute of Electrical and Electronics Engineers) style.
   * Used in engineering and computer science, with specific author and title formatting.
   * Format: F. Last, "Article Title," Journal Name, vol. #, no. #, pp. ##-##, Year.
   */
  renderIEEEReference() {
    if (this.text) {
      return html`<div class="mt-3">[${this.appearanceIndex ?? 1}] ${this.text}</div>`
    }

    const index = this.appearanceIndex ?? 1
    const authors = formatIEEEAuthors(this.authors)
    const title = this.workType === 'Article' 
      ? html`"<slot name="title"></slot>," `
      : html`<em><slot name="title"></slot></em>, `
    
    const journal = this.isPartOf ? html`<em>${getTitle(this.isPartOf)}</em>` : ''
    const volume = this.isPartOf?.volumeNumber ? html`, vol. ${this.isPartOf.volumeNumber}` : ''
    const issue = this.isPartOf?.issueNumber ? html`, no. ${this.isPartOf.issueNumber}` : ''
    const pages = formatPagesIEEE(this.pageStart, this.pageEnd, this.pagination)
    const date = this.date ? html`, ${dateYear(this.date)}` : ''
    const link = getLink(this)

    return html`<div class="mt-3">
      [${index}] ${authors}${title}${journal}${volume}${issue}${pages}${date}${link}.
    </div>`
  }

  /*
   * Render full reference in Harvard referencing style.
   * Emphasizes author and year, commonly used in business and social sciences.
   * Format: Last, F. Year. 'Article Title'. Journal Name, Volume(Issue), pp. ##-##.
   */
  renderHarvardReference() {
    if (this.text) {
      return html`<div class="mt-3">${this.text}</div>`
    }

    const authors = formatHarvardAuthors(this.authors)
    const year = this.date ? ` ${dateYear(this.date)}. ` : '. '
    const title = this.workType === 'Article' 
      ? html`'<slot name="title"></slot>'. `
      : html`<em><slot name="title"></slot></em>. `
    
    const journal = this.isPartOf ? html`<em>${getTitle(this.isPartOf)}</em>` : ''
    const volume = this.isPartOf?.volumeNumber ? html`, ${this.isPartOf.volumeNumber}` : ''
    const issue = this.isPartOf?.issueNumber ? html`(${this.isPartOf.issueNumber})` : ''
    const pages = formatPagesHarvard(this.pageStart, this.pageEnd, this.pagination)
    const link = getLink(this)

    return html`<div class="mt-3">
      ${authors}${year}${title}${journal}${volume}${issue}${pages}${link}.
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
 * Format page ranges with en-dash between start and end pages.
 * Handles various page formats: single pages, ranges, or custom pagination strings.
 * Returns empty string if no page information available.
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

/*
 * Extract plain text title from structured inline content.
 * Recursively processes inline elements to build complete title string.
 * Handles various inline types including text nodes and nested content.
 */
function getTitle(ref: ReferenceType): string {
  if (ref.title) {
    function inlineToString(inline: Inline): string {
      if (typeof inline === 'string') return inline
      if (typeof inline === 'number') return inline.toString()
      if (typeof inline === 'boolean') return inline.toString()
      if (inline && typeof inline === 'object' && 'type' in inline) {
        if (inline.type === 'Text') {
          return (inline as Text).value.string || ''
        } else if ('content' in inline) {
          return (inline.content as Inline[]).map((inline) => inlineToString(inline)).join('')
        }
      }
      return ''
    }
    return ref.title.map(inlineToString).join('')
  }
  return ''
}

/*
 * Format author list according to APA style guidelines.
 * Uses "Last, F. M." format with ampersand before final author.
 * Handles multiple authors with proper punctuation and conjunction.
 */
function formatAPAAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const formatted = authors.map(author => {
    switch (author.type) {
      case 'Person':
        if (author.familyNames?.length && author.givenNames?.length > 0) {
          return author.familyNames.join(' ') + ', ' + 
                 author.givenNames.map(name => `${name[0]}.`).join(' ')
        }
        return author.name || 'Anon'
      case 'Organization':
      case 'SoftwareApplication':
        return author.name || 'Anon'
      case 'AuthorRole':
        switch (author.author.type) {
          case 'Person':
          case 'Organization':
          case 'SoftwareApplication':
            return formatAPAAuthors([author.author as Author])
          case 'Thing':
            return author.author.name || 'Anon'
        }
    }
  })
  
  if (formatted.length === 1) return formatted[0]
  if (formatted.length === 2) return `${formatted[0]}, & ${formatted[1]}`
  return formatted.slice(0, -1).join(', ') + ', & ' + formatted[formatted.length - 1]
}

/*
 * Extract journal title for APA reference formatting.
 * Simple wrapper around getTitle for consistency with other APA functions.
 */
function formatJournalAPA(ref: ReferenceType): string {
  return getTitle(ref)
}

/*
 * Format page numbers for APA style with preceding comma.
 * Returns empty string if no pages, otherwise adds comma prefix.
 */
function formatPagesAPA(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `, ${pages}` : ''
}

/*
 * Format author list according to MLA style guidelines.
 * First author as "Last, First", subsequent authors as "First Last".
 * Uses "and" before final author, "et al." for more than two authors.
 */
function formatMLAAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const first = authors[0]
  let result = ''
  
  switch (first.type) {
    case 'Person':
      if (first.familyNames?.length && first.givenNames?.length > 0) {
        result = first.familyNames.join(' ') + ', ' + first.givenNames.join(' ')
      } else {
        result = first.name || 'Anon'
      }
      break
    case 'Organization':
    case 'SoftwareApplication':
      result = first.name || 'Anon'
      break
    case 'AuthorRole':
      switch (first.author.type) {
        case 'Person':
          if (first.author.familyNames?.length && first.author.givenNames?.length > 0) {
            result = first.author.familyNames.join(' ') + ', ' + first.author.givenNames.join(' ')
          } else {
            result = first.author.name || 'Anon'
          }
          break
        default:
          result = first.author.name || 'Anon'
      }
      break
  }
  
  if (authors.length === 2) {
    const second = authors[1]
    let secondName = 'Anon'
    switch (second.type) {
      case 'Person':
        if (second.givenNames?.length && second.familyNames?.length) {
          secondName = `${second.givenNames.join(' ')} ${second.familyNames.join(' ')}`
        } else {
          secondName = second.name || 'Anon'
        }
        break
      case 'Organization':
      case 'SoftwareApplication':
        secondName = second.name || 'Anon'
        break
      case 'AuthorRole':
        secondName = second.author.name || 'Anon'
        break
    }
    result += ` and ${secondName}`
  } else if (authors.length > 2) {
    result += ' et al.'
  }
  
  return result + '. '
}

/*
 * Format page numbers for MLA style with "pp." prefix.
 * MLA requires page abbreviation before page numbers.
 */
function formatPagesMLA(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `, pp. ${pages}` : ''
}

/*
 * Format author list according to Chicago Manual of Style.
 * Similar to MLA but with specific punctuation requirements.
 * First author inverted, subsequent authors in natural order.
 */
function formatChicagoAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const first = authors[0]
  let result = ''
  
  switch (first.type) {
    case 'Person':
      if (first.familyNames?.length && first.givenNames?.length > 0) {
        result = first.familyNames.join(' ') + ', ' + first.givenNames.join(' ')
      } else {
        result = first.name || 'Anon'
      }
      break
    case 'Organization':
    case 'SoftwareApplication':
      result = first.name || 'Anon'
      break
    case 'AuthorRole':
      switch (first.author.type) {
        case 'Person':
          if (first.author.familyNames?.length && first.author.givenNames?.length > 0) {
            result = first.author.familyNames.join(' ') + ', ' + first.author.givenNames.join(' ')
          } else {
            result = first.author.name || 'Anon'
          }
          break
        default:
          result = first.author.name || 'Anon'
      }
      break
  }
  
  if (authors.length === 2) {
    const second = authors[1]
    let secondName = 'Anon'
    switch (second.type) {
      case 'Person':
        if (second.givenNames?.length && second.familyNames?.length) {
          secondName = `${second.givenNames.join(' ')} ${second.familyNames.join(' ')}`
        } else {
          secondName = second.name || 'Anon'
        }
        break
      case 'Organization':
      case 'SoftwareApplication':
        secondName = second.name || 'Anon'
        break
      case 'AuthorRole':
        secondName = second.author.name || 'Anon'
        break
    }
    result += ` and ${secondName}`
  } else if (authors.length > 2) {
    result += ' et al.'
  }
  
  return result + '. '
}

/*
 * Format page numbers for Chicago style with colon prefix.
 * Chicago style uses colon to separate publication info from pages.
 */
function formatPagesChicago(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `: ${pages}` : ''
}

/*
 * Format author list according to Vancouver/ICMJE medical journal style.
 * Uses "Last F" format without periods, limits to 6 authors before "et al.".
 * Designed for space-efficient medical and scientific citations.
 */
function formatVancouverAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const formatted = authors.map(author => {
    switch (author.type) {
      case 'Person':
        if (author.familyNames?.length && author.givenNames?.length > 0) {
          return author.familyNames.join(' ') + ' ' + 
                 author.givenNames.map(name => name[0]).join('')
        }
        return author.name || 'Anon'
      case 'Organization':
      case 'SoftwareApplication':
        return author.name || 'Anon'
      case 'AuthorRole':
        switch (author.author.type) {
          case 'Person':
          case 'Organization':
          case 'SoftwareApplication':
            return formatVancouverAuthors([author.author as Author])
          case 'Thing':
            return author.author.name || 'Anon'
        }
    }
  })
  
  if (formatted.length <= 6) {
    return formatted.join(', ') + '. '
  } else {
    return formatted.slice(0, 3).join(', ') + ', et al. '
  }
}

/*
 * Format page numbers for Vancouver style with colon prefix, no space.
 * Vancouver uses compressed format: "Journal. 2020;15(3):123-8."
 */
function formatPagesVancouver(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `:${pages}` : ''
}

/*
 * Format author list according to IEEE engineering journal style.
 * Uses "F. M. Last" format with "and" before final author.
 * Includes trailing comma after complete author list.
 */
function formatIEEEAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const formatted = authors.map(author => {
    switch (author.type) {
      case 'Person':
        if (author.givenNames?.length && author.familyNames?.length) {
          return author.givenNames.map(name => `${name[0]}.`).join(' ') + ' ' + 
                 author.familyNames.join(' ')
        }
        return author.name || 'Anon'
      case 'Organization':
      case 'SoftwareApplication':
        return author.name || 'Anon'
      case 'AuthorRole':
        switch (author.author.type) {
          case 'Person':
          case 'Organization':
          case 'SoftwareApplication':
            return formatIEEEAuthors([author.author as Author])
          case 'Thing':
            return author.author.name || 'Anon'
        }
    }
  })
  
  if (formatted.length === 1) return formatted[0] + ', '
  if (formatted.length === 2) return `${formatted[0]} and ${formatted[1]}, `
  return formatted.slice(0, -1).join(', ') + ', and ' + formatted[formatted.length - 1] + ', '
}

/*
 * Format page numbers for IEEE style with "pp." abbreviation.
 * IEEE uses standard academic page formatting with comma prefix.
 */
function formatPagesIEEE(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `, pp. ${pages}` : ''
}

/*
 * Format author list according to Harvard referencing style.
 * Uses "Last, F." format with ampersand before final author.
 * Similar to APA but without trailing comma in final conjunction.
 */
function formatHarvardAuthors(authors?: Author[]): string {
  if (!authors || authors.length === 0) return 'Anon'
  
  const formatted = authors.map(author => {
    switch (author.type) {
      case 'Person':
        if (author.familyNames?.length && author.givenNames?.length > 0) {
          return author.familyNames.join(' ') + ', ' + 
                 author.givenNames.map(name => `${name[0]}.`).join('')
        }
        return author.name || 'Anon'
      case 'Organization':
      case 'SoftwareApplication':
        return author.name || 'Anon'
      case 'AuthorRole':
        switch (author.author.type) {
          case 'Person':
          case 'Organization':
          case 'SoftwareApplication':
            return formatHarvardAuthors([author.author as Author])
          case 'Thing':
            return author.author.name || 'Anon'
        }
    }
  })
  
  if (formatted.length === 1) return formatted[0]
  if (formatted.length === 2) return `${formatted[0]} & ${formatted[1]}`
  return formatted.slice(0, -1).join(', ') + ' & ' + formatted[formatted.length - 1]
}

/*
 * Format page numbers for Harvard style with "pp." abbreviation and comma.
 * Follows standard academic conventions with page abbreviation.
 */
function formatPagesHarvard(pageStart?: string, pageEnd?: string, pagination?: string): string {
  const pages = pagesEndashed(pageStart, pageEnd, pagination)
  return pages ? `, pp. ${pages}` : ''
}
