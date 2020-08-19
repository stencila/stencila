import { after, append, create, select } from '../../../util'
import { getArticlePdfUrl, getFiguresPdfUrl } from './dataProvider'
import { articleData } from './query'

const deriveUrl = (type: string, id: string): string => {
  switch (type) {
    case 'executable-version':
      return `https://elifesciences.org/articles/${id}/executable/download`
    case 'bibtex':
      return `https://elifesciences.org/articles/${id}.bib`
    case 'ris':
      return `https://elifesciences.org/articles/${id}.ris`
    case 'mendeley':
      return `https://www.mendeley.com/import?doi=10.7554/eLife.${id}`
    case 'readcube':
      return `https://www.readcube.com/articles/10.7554/eLife.${id}`
  }
  return ''
}

const buildLinkToFiguresPdf = (url: string): void => {
  after(
    select('[data-is-download-pdf-list-item]')[0],
    create('li', null, createSimpleLink(url, 'Figures PDF'))
  )
}

const buildMenu = (
  articleId: string,
  articleTitle: string,
  pdfUrl: string,
  menuId: string
): void => {
  after(
    select(':--references')[0],
    create(
      'section',
      { id: menuId, class: 'downloads' },

      create('h2', null, 'Download links'),
      create('h3', null, 'Downloads'),
      create(
        'ul',
        null,
        create(
          'li',
          { 'data-is-download-pdf-list-item': true },
          createSimpleLink(pdfUrl, 'Article PDF')
        ),
        create(
          'li',
          null,
          createSimpleLink(
            deriveUrl('executable-version', articleId),
            'Executable version'
          ),
          create(
            'div',
            { class: 'downloads--link' },
            createSimpleLink(
              'https://elifesciences.org/labs/7dbeb390/reproducible-document-stack-supporting-the-next-generation-research-article',
              'What are executable versions?'
            )
          )
        )
      ),
      create('h3', null, 'Download citations'),
      create(
        'ul',
        null,
        create(
          'li',
          null,
          createSimpleLink(deriveUrl('bibtex', articleId), 'BibTeX')
        ),
        create('li', null, createSimpleLink(deriveUrl('ris', articleId), 'RIS'))
      ),
      create('h3', null, 'Open citations'),
      create(
        'ul',
        null,
        create(
          'li',
          null,
          createSimpleLink(deriveUrl('mendeley', articleId), 'Mendeley')
        ),
        create(
          'li',
          null,
          createSimpleLink(deriveUrl('readcube', articleId), 'ReadCube')
        )
      )
    )
  )
}

const buildLinkToMenu = (contentHeader: Element, menuId: string): void => {
  const text =
    'A two-part list of links to download the article, or parts of the article, in various formats.'
  append(
    contentHeader,
    create(
      'a',
      { href: `#${menuId}`, class: 'download-link' },
      create('span', { class: 'download-link-text' }, text)
    )
  )
}

const createSimpleLink = (href: string, text: string): Element =>
  create('a', { href, target: '_parent' }, text)

export const build = (
  contentHeader: Element,
  articleTitle: string,
  articleId: string,
  article: articleData
): void => {
  const menuId = 'downloadMenu'
  const figuresPdf = getFiguresPdfUrl(article)
  buildMenu(articleId, articleTitle, getArticlePdfUrl(article), menuId)
  if (figuresPdf !== '') {
    buildLinkToFiguresPdf(getFiguresPdfUrl(article))
  }
  buildLinkToMenu(contentHeader, menuId)
}
