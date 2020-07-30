import { append, create } from '../../../util'
import { getCopyrightLicense } from './dataProvider'
import { articleData } from './query'

const deriveUrl = (id: string): string => {
  switch (id) {
    case 'CC0-1.0':
      return 'https://creativecommons.org/publicdomain/zero/1.0/'
    case 'CC-BY-1.0':
      return 'https://creativecommons.org/licenses/by/1.0/'
    case 'CC-BY-2.0':
      return 'https://creativecommons.org/licenses/by/2.0/'
    case 'CC-BY-2.5':
      return 'https://creativecommons.org/licenses/by/2.5/'
    case 'CC-BY-3.0':
      return 'https://creativecommons.org/licenses/by/3.0/'
    case 'CC-BY-4.0':
      return 'https://creativecommons.org/licenses/by/4.0/'
  }
  return ''
}

const buildMenu = (contentHeader: Element, license: string): void => {
  append(
    contentHeader,
    create(
      'ul',
      { class: 'content-header__icons' },
      create(
        'li',
        null,
        create(
          'a',
          {
            class: 'content-header__icon content-header__icon--oa',
            href: 'https://en.wikipedia.org/wiki/Open_access',
            target: '_parent',
          },
          create('span', { class: 'visuallyhidden' }, 'Open access')
        )
      ),
      create(
        'li',
        null,
        create(
          'a',
          {
            class: 'content-header__icon content-header__icon--cc',
            href: deriveUrl(license),
            target: '_parent',
          },
          create('span', { class: 'visuallyhidden' }, 'Copyright information')
        )
      )
    )
  )
}

export const build = (contentHeader: Element, article: articleData): void => {
  try {
    buildMenu(contentHeader, getCopyrightLicense(article))
  } catch (err) {
    console.error(err)
  }
}
