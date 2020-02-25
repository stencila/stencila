import { after, create, replace, select, ready, attr } from '../../util'

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
  // This is a proposal, but due to conflicts with existing styles, is currently
  // not enabled.
  return
  select(':--Person :--name').forEach(span => {
      const name = attr(span, 'content')
      const givenNames = select(span, ':--givenName')
        .map(item => item.textContent)
        .join(' ')
      const familyNames = select(span, ':--familyName')
        .map(item => item.textContent)
        .join(' ')

      replace(
        span,
        create(`meta :--name [content=${name}]`),
        create(
          'ol:-givenNames',
          ...givenNames
            .split(/\s+/)
            .map(givenName => create('li :--givenName', givenName))
        ),
        create(
          'ol:-familyNames',
          ...familyNames
            .split(/\s+/)
            .map(familyName => create('li :--familyName', familyName))
        )
      )
  })
})
