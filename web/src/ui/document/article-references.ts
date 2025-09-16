import { MutationController } from '@lit-labs/observers/mutation-controller'
import { Author } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { Reference } from '../../nodes/reference'
import { withTwind } from '../../twind'

@customElement('stencila-ui-article-references')
@withTwind()
export class ArticleReferences extends LitElement {
  @property({ attribute: 'sort-mode' })
  sortMode: 'alphabetic' | 'numeric' = 'alphabetic'

  /**
   * A mutation controller used to determine whether to add a "References" heading
   *
   * @see onSlotChange
   */
  // @ts-expect-error is never read
  private mutationController: MutationController

  /**
   * Initialize the mutation controller when the slot changes
   */
  onSlotChange({ target: slot }: Event) {
    const referencesElem = (slot as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]
    if (!referencesElem) {
      return
    }

    this.mutationController = new MutationController(this, {
      target: referencesElem,
      config: {
        childList: true,
      },
      callback: () => {
        const references = Array.from(referencesElem.querySelectorAll('stencila-reference')) as Reference[]

        if (references.length > 0) {
          // Add heading if not already present
          if (referencesElem.querySelectorAll('h1').length == 0) {
            const heading = document.createElement('stencila-heading')
            heading.setAttribute('depth', '1')
            heading.setAttribute('level', '1')
            heading.innerHTML = '<h1 slot="content">References</h1>'
            referencesElem.prepend(heading)
          }

          // Sort references based on current sort mode
          this.sortReferences(referencesElem, references)
        }
      },
    })
  }

  override render() {
    return html`<slot @slotchange=${this.onSlotChange}></slot>`
  }

  /*
   * Sort references using CSS order property instead of DOM manipulation.
   * This avoids the need to physically reorder elements and prevents recursion.
   */
  private sortReferences(container: Element, references: Reference[]): void {
    if (references.length <= 1) return // No need to sort single or no references

    // Sort references based on current mode to get the desired order
    const sortedReferences = this.sortMode === 'numeric'
      ? this.sortNumerically([...references])
      : this.sortAlphabetically([...references])

    // Apply CSS order property to achieve the desired visual order
    sortedReferences.forEach((ref, index) => {
      (ref as HTMLElement).style.order = index.toString()
    });

    // Ensure the container uses flexbox to respect the order property
    (container as HTMLElement).style.display = 'flex';
    (container as HTMLElement).style.flexDirection = 'column'

    // Ensure heading has order -1 to stay at top
    const heading = container.querySelector('stencila-heading') as HTMLElement
    if (heading) {
      heading.style.order = '-1'
    }
  }

  /*
   * Extract the first author's surname for alphabetical sorting.
   * Handles Person, Organization, AuthorRole types like the reference component.
   */
  private getFirstAuthorSurname(reference: Reference): string {
    const authors = reference.authors as Author[] | undefined

    if (!authors || authors.length === 0) {
      return 'Anon'
    }

    const firstAuthor = authors[0]
    switch (firstAuthor.type) {
      case 'Person':
        return firstAuthor.familyNames?.[0] ?? firstAuthor.name ?? 'Anon'
      case 'Organization':
      case 'SoftwareApplication':
        return firstAuthor.name ?? 'Anon'
      case 'AuthorRole':
        switch (firstAuthor.author.type) {
          case 'Person':
            return firstAuthor.author.familyNames?.[0] ?? firstAuthor.author.name ?? 'Anon'
          case 'Organization':
          case 'SoftwareApplication':
          case 'Thing':
            return firstAuthor.author.name ?? 'Anon'
        }
        break
      default:
        return 'Anon'
    }
  }

  /*
   * Extract year from reference element for secondary sorting.
   * Returns empty string for missing dates to sort them last.
   */
  private getYear(reference: Reference): string {
    const date = reference.date as string | undefined
    return date?.slice(0, 4) ?? ''
  }

  /*
   * Extract title text from reference element for tertiary sorting.
   * Gets the text content from the title slot.
   */
  private getTitle(reference: Reference): string {
    const titleSlot = reference.querySelector('[slot="title"]')
    return titleSlot?.textContent?.trim() ?? ''
  }

  /*
   * Extract appearance index for numeric sorting.
   * Returns very high number for missing indices to sort them last.
   */
  private getAppearanceIndex(reference: Reference): number {
    return reference.appearanceIndex ?? 999999
  }

  /*
   * Sort references alphabetically by author surname, then year, then title.
   * Follows standard academic bibliography conventions for multi-level sorting.
   */
  private sortAlphabetically(references: Reference[]): Reference[] {
    return references.sort((a, b) => {
      // Primary sort: Author surname
      const surnameA = this.getFirstAuthorSurname(a).toLowerCase()
      const surnameB = this.getFirstAuthorSurname(b).toLowerCase()
      if (surnameA !== surnameB) {
        return surnameA.localeCompare(surnameB)
      }

      // Secondary sort: Year (oldest first, empty years last)
      const yearA = this.getYear(a)
      const yearB = this.getYear(b)
      if (yearA !== yearB) {
        // Empty years go last
        if (!yearA) return 1
        if (!yearB) return -1
        return yearA.localeCompare(yearB)
      }

      // Tertiary sort: Title
      const titleA = this.getTitle(a).toLowerCase()
      const titleB = this.getTitle(b).toLowerCase()
      return titleA.localeCompare(titleB)
    })
  }

  /*
   * Sort references numerically by appearance index.
   * Used for citation styles that reference by order of appearance.
   */
  private sortNumerically(references: Reference[]): Reference[] {
    return references.sort((a, b) => {
      return this.getAppearanceIndex(a) - this.getAppearanceIndex(b)
    })
  }
}
