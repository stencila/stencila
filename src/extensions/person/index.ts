import { after, create, replace, select, ready } from '../../scripts/dom'

/**
 * Currently, Encoda encodes the names of a `Person` within a `span` e.g.
 *
 * ```html
 * <span itemprop="name" content="Sarel J. Fleishman">
 *   <span itemprop="givenName">Sarel J.</span>
 *   <span itemprop="familyName">Fleishman</span>
 * </span>
 * ```
 *
 * Note: the `itemprop` name with `content` ensures
 * conformance with GSDTT.
 *
 * So that we can style the given names and family names,
 * and consistent with other array properties, a better encoding would be:
 *
 * ```html
 * <meta itemprop="name" content="Sarel J. Fleishman">
 * <ol data-itemprop="givenNames">
 *   <li itemprop="givenName">Sarel</li>
 *   <li itemprop="givenName">J.</li>
 * </ol>
 * <ol data-itemprop="familyNames">
 *   <li itemprop="familyName">Fleishman</li>
 * </ol>
 * ```
 */
ready(() => {
  return
  select('[itemtype="http://schema.org/Person"] span[itemprop=name]').forEach(
    span => {
      const name = span.getAttribute('content')
      const givenNames = select(span, '[itemprop=givenName]')
        .map(item => item.textContent)
        .join(' ')
      const familyNames = select(span, '[itemprop=familyName]')
        .map(item => item.textContent)
        .join(' ')

      after(
        span,
        create(
          'ol[data-itemprop=givenNames]',
          ...givenNames
            .split(/\s+/)
            .map(givenName => create('li[itemprop=givenName]', givenName))
        ),
        create(
          'ol[data-itemprop=familyNames]',
          ...familyNames
            .split(/\s+/)
            .map(familyName => create('li[itemprop=familyName]', familyName))
        )
      )
      replace(span, `meta[itemprop=name][content=${name}]`)
    }
  )
})
