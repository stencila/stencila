import { after, before, create, first, select } from '../../../util'
import { getArticlePdfUrl, getFiguresPdfUrl } from './eLifeDataProvider'

const getUrl = (type: string, id: string, title = ''): string => {
  switch (type) {
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

const addFiguresPdfUrl = (url: string): void => {
  after(
    select('[data-is-download-pdf-link]')[0],
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
          null,
          create(
            'a',
            { href: pdfUrl, 'data-is-download-pdf-link': true },
            'Article PDF'
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
          create('a', { href: `${getUrl('bibtex', articleId)}` }, 'BibTeX')
        ),
        create(
          'li',
          null,
          create('a', { href: `${getUrl('ris', articleId)}` }, 'RIS')
        )
      ),
      create('h3', null, 'Open citations'),
      create(
        'ul',
        null,
        create(
          'li',
          null,
          create('a', { href: `${getUrl('mendeley', articleId)}` }, 'Mendeley')
        ),
        create(
          'li',
          null,
          create('a', { href: `${getUrl('readcube', articleId)}` }, 'ReadCube')
        ),
        create(
          'li',
          null,
          create(
            'a',
            { href: `${getUrl('papers', articleId, articleTitle)}` },
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

export const build = (articleId: string, articleTitle: string): void => {
  const menuId = 'downloadMenu'
  try {
    getArticlePdfUrl(articleId)
      .then((pdfUri) => buildMenu(articleId, articleTitle, pdfUri, menuId))
      .then(() => getFiguresPdfUrl(articleId))
      .then((figuresPdfUrl: string) => addFiguresPdfUrl(figuresPdfUrl))
      .then(() => buildLinkToMenu(menuId))
      .catch((err: Error) => {
        throw err
      })
  } catch (err) {
    console.error(err)
  }
}
