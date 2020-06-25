import { after, before, create, first, select } from '../../../util'
import { getArticlePdfUrl, getFiguresPdfUrl } from './dataProvider'

const deriveUrl = (type: string, id: string, title = ''): string => {
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
    case 'papers':
      return `papers2://url/https%3A%2F%2Felifesciences.org%2Farticles%2F${id}?title=${encodeURIComponent(
        title
      )}`
  }
  return ''
}

const buildLinkToFiguresPdf = (url: string): void => {
  after(
    select('[data-is-download-pdf-list-item]')[0],
    create('li', null, create('a', { href: url }, 'Figures PDF'))
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
          create('a', { href: pdfUrl }, 'Article PDF')
        ),
        create(
          'li',
          null,
          create(
            'a',
            { href: `${deriveUrl('executable-version', articleId)}` },
            'Executable version'
          ),
          create(
            'div',
            { class: 'downloads--link' },
            create(
              'a',
              {
                href:
                  'https://preview--journal.elifesciences.org/labs/7dbeb390',
              },
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
          create('a', { href: `${deriveUrl('bibtex', articleId)}` }, 'BibTeX')
        ),
        create(
          'li',
          null,
          create('a', { href: `${deriveUrl('ris', articleId)}` }, 'RIS')
        )
      ),
      create('h3', null, 'Open citations'),
      create(
        'ul',
        null,
        create(
          'li',
          null,
          create(
            'a',
            { href: `${deriveUrl('mendeley', articleId)}` },
            'Mendeley'
          )
        ),
        create(
          'li',
          null,
          create(
            'a',
            { href: `${deriveUrl('readcube', articleId)}` },
            'ReadCube'
          )
        ),
        create(
          'li',
          null,
          create(
            'a',
            { href: `${deriveUrl('papers', articleId, articleTitle)}` },
            'Papers'
          )
        )
      )
    )
  )
}

const buildLinkToMenu = (menuId: string): Promise<unknown> => {
  const url = `#${menuId}`
  const text =
    'A two-part list of links to download the article, or parts of the article, in various formats.'
  const articleTitle = first(':--Article > :--title')
  if (articleTitle === null) {
    return Promise.reject(
      new Error("Can't find element to bolt the download link on top of")
    )
  }
  before(
    articleTitle,
    create(
      'div',
      { class: 'download-link-wrapper' },
      create(
        'a',
        { href: url, class: 'download-link' },
        create('span', { class: 'download-link-text' }, text)
      )
    )
  )
  return Promise.resolve()
}

export const build = (articleTitle: string, articleId: string): void => {
  const menuId = 'downloadMenu'
  try {
    getArticlePdfUrl(articleId)
      .then((pdfUri) => buildMenu(articleId, articleTitle, pdfUri, menuId))
      .then(() => getFiguresPdfUrl(articleId))
      .then((figuresPdfUrl: string) => buildLinkToFiguresPdf(figuresPdfUrl))
      .then(() => buildLinkToMenu(menuId))
      .catch((err: Error) => {
        throw err
      })
  } catch (err) {
    console.error(err)
  }
}
