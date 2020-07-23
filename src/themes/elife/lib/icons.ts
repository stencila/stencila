import { append, create } from '../../../util'
import { getCopyrightLicense } from './dataProvider'

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

const buildMenu = (
  contentHeader: Element,
  license: string
): Promise<unknown> => {
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
  return Promise.resolve()
}

export const build = (contentHeader: Element, articleId: string): void => {
  try {
    getCopyrightLicense(articleId)
      .then((license: string) => buildMenu(contentHeader, license))
      .catch((err: Error) => {
        throw err
      })
  } catch (err) {
    console.error(err)
  }
}
