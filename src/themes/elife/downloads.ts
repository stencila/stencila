import { append, create, select } from '../../util'

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
      return `papers2://url/https%3A%2F%2Felifesciences.org%2Farticles%2F46206?title=${encodeURIComponent(
        title
      )}`
  }
  return ''
}

export const build = (articleId: string, articleTitle: string): void => {
  append(
    select(':--references')[0],
    create('h2', null, 'Download links'),
    create('h3', null, 'Downloads'),
    create(
      'ul',
      null,
      create('li', null, create('a', { href: '#' }, 'Article PDF')),
      create('li', null, create('a', { href: '#' }, 'Figures PDF'))
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
}
