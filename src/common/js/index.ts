/**
 * This file is imported by all themes, allowing for shared configurations among themes.
 */

import { codeHighlight } from './syntaxHighlight'

export const formatReferences = (): void => {
  const referenceListItemSel = '.references > li'
  const titleSel = 'span[itemprop="headline"]'
  const datePublishedSel = '[itemprop="datePublished"]'
  const publicationIssueSel = '[itemtype="https://schema.org/PublicationIssue"]'

  document.querySelectorAll(referenceListItemSel).forEach(node => {
    const datePublished = node.querySelector(datePublishedSel) as HTMLElement
    const title = node.querySelector(titleSel) as HTMLElement

    if (title) {
      const titleCopy = title.cloneNode(true) as HTMLElement

      // Add title node after original datePublished node
      if (datePublished && datePublished.parentNode) {
        datePublished.parentNode.insertBefore(
          titleCopy,
          datePublished.nextSibling
        )
      }

      // Add datePublished node after PublicationIssue node (if exists)
      const publicationIssue = node.querySelector(
        publicationIssueSel
      ) as HTMLElement
      const datePublishedCopy = datePublished.cloneNode(true) as HTMLElement

      if (publicationIssue && publicationIssue.parentNode) {
        publicationIssue.parentNode.insertBefore(
          datePublishedCopy,
          publicationIssue.nextSibling
        )
      } else {
        // Otherwise, add node after titleCopy
        if (titleCopy && titleCopy.parentNode) {
          titleCopy.parentNode.insertBefore(
            datePublishedCopy,
            titleCopy.nextSibling
          )
        }
      }
    }
  })
}

const onReadyHandler = (): void => {
  codeHighlight()
  // Use setTimeout to queue formatReferences until
  // the current call stack gets executed (allow DOM elements
  // to load before rearranging references for theme styles)
  window.setTimeout(formatReferences, 0)
}

export const load = (): void => {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', onReadyHandler)
  } else {
    onReadyHandler()
  }
}
