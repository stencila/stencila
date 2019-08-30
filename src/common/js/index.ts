/**
 * This file is imported by all themes, allowing for shared configurations among themes.
 */

import './syntaxHighlight'

const ready = (): void => {
  const themeSelect = document.querySelector('#theme-select')

  if (themeSelect) {
    themeSelect.addEventListener('change', event => {
      const element = event.target as HTMLInputElement
      const theme = element.value

      document
        .querySelectorAll('link[rel="stylesheet"]')
        .forEach(node =>
          node.id === theme
            ? ((<HTMLInputElement>node).disabled = false)
            : ((<HTMLInputElement>node).disabled = true)
        )
    })
  }

  const referenceListItemSel = '[itemprop="references"] > li'
  const titleSel = '[itemprop="title"]'
  const datePublishedSel = '[itemprop="datePublished"]'
  const publicationIssueSel =
    '[itemtype="https://schema.stenci.la/PublicationIssue"]'

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

document.addEventListener('DOMContentLoaded', ready)
