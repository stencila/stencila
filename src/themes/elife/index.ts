import '../../extensions/headings'
import '../../extensions/person'
// import { first, ready } from '../../util'
import * as util from '../../util'
import DateTimeFormat = Intl.DateTimeFormat

const dateFormatter = new DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric'
})

const formatDate = (date: Date): string => {
  return dateFormatter.format(date)
}

const buildDownloads = (): void => {
  util.append(
    util.select(':--references')[0],
    util.create('h2', null, 'Download links'),
    util.create('h3', null, 'Downloads'),
    util.create(
      'ul',
      null,
      util.create('li', null, util.create('a', { href: '#' }, 'Article PDF')),
      util.create('li', null, util.create('a', { href: '#' }, 'Figures PDF'))
    ),
    util.create('h3', null, 'Download citations'),
    util.create(
      'ul',
      null,
      util.create('li', null, util.create('a', { href: '#' }, 'BibTeX')),
      util.create('li', null, util.create('a', { href: '#' }, 'RIS'))
    ),
    util.create('h3', null, 'Open citations'),
    util.create(
      'ul',
      null,
      util.create('li', null, util.create('a', { href: '#' }, 'Mendeley')),
      util.create('li', null, util.create('a', { href: '#' }, 'ReadCube')),
      util.create('li', null, util.create('a', { href: '#' }, 'Papers')),
      util.create('li', null, util.create('a', { href: '#' }, 'CiteULike'))
    )
  )
}

util.ready((): void => {
  const dateEl = util.first(':--Date')
  if (!(dateEl instanceof Element)) return
  const date = new Date(dateEl.innerHTML)
  dateEl.innerHTML = formatDate(date)

  buildDownloads()
})
