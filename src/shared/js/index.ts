/**
 * This file is imported by all themes, allowing for shared configurations among themes.
 */

export const formatReferences = (): void => {
  const referenceListItemSel =
    '[data-itemprop="references"] > [itemprop="citation"]'
  const titleSel = '[itemprop="headline"]'
  const datePublishedSel = '[itemprop="datePublished"]'
  const publicationIssueSel = '[itemtype="https://schema.org/PublicationIssue"]'

  document.querySelectorAll(referenceListItemSel).forEach(node => {
    const datePublished = node.querySelector(datePublishedSel)
    const title = node.querySelector(titleSel)

    if (title !== null) {
      const titleCopy = title.cloneNode(true)

      // Add title node after original datePublished node
      if (datePublished !== null && datePublished.parentNode !== null) {
        datePublished.parentNode.insertBefore(
          titleCopy,
          datePublished.nextSibling
        )
      }

      // Add datePublished node after PublicationIssue node (if exists)
      const publicationIssue = node.querySelector(publicationIssueSel)
      const datePublishedCopy = datePublished?.cloneNode(true)

      if (
        datePublishedCopy !== undefined &&
        publicationIssue !== null &&
        publicationIssue.parentNode !== null
      ) {
        publicationIssue.parentNode.insertBefore(
          datePublishedCopy,
          publicationIssue.nextSibling
        )
      } else if (datePublishedCopy !== undefined) {
        // Otherwise, add node after titleCopy
        if (titleCopy !== null && titleCopy.parentNode !== null) {
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
  // Use setTimeout to queue formatReferences until
  // the current call stack gets executed (allow DOM elements
  // to load before rearranging references for theme styles)
  window.setTimeout(formatReferences, 0)
}

export const load = (): void => {
  // Do not do anything if not in the browser
  // (e.g. when loading themes in Node.js)
  if (typeof window === 'undefined') return

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', onReadyHandler)
  } else {
    onReadyHandler()
  }
}
