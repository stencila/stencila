import '../../common/js/index'

const ready = (): void => {
  const referenceListItemSel = '[itemprop="references"] > li'
  const datePublishedSel = '[itemprop="datePublished"]'
  const publicationIssueTitleSel =
    '[itemtype="https://schema.stenci.la/PublicationIssue"] [itemprop="title"]'

  document.querySelectorAll(referenceListItemSel).forEach(node => {
    // If PublicationIssue title node exists,
    const publicationIssueTitle = node.querySelector(publicationIssueTitleSel)

    // Move last datePublished after PublicationIssue title.
    const datePublished = document.querySelectorAll(datePublishedSel)
    const lastDatePublished = datePublished[datePublished.length - 1]

    if (publicationIssueTitle && publicationIssueTitle.parentNode) {
      publicationIssueTitle.parentNode.insertBefore(
        lastDatePublished,
        publicationIssueTitle.nextSibling
      )
    }
  })
}

document.addEventListener('DOMContentLoaded', ready)
