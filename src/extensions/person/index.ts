import { create, replace, select, ready, attr } from '../../util'

/**
 * Temporary DOM restructuring for `Person` nodes. Associated issue
 * in Encoda: https://github.com/stencila/encoda/issues/454
 *
 * Currently, Encoda encodes the names of a `Person` within a `span` e.g. an author
 * of an `Article`:
 *
 * ```html
 * <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
 *    <span itemprop="name" content="Sarel J. Fleishman">
 *      <span itemprop="givenName">Sarel J.</span>
 *      <span itemprop="familyName">Fleishman</span>
 *    </span>
 * </li>
 * ```
 *
 * Note: the `itemprop` name with `content` ensures
 * conformance with GSDTT.
 *
 * So that we can style the given names and family names,
 * and consistent with other array properties, a better encoding would be:
 *
 * ```html
 * <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
 *   <meta itemprop="name" content="Sarel J. Fleishman">
 *   <span data-itemprop="givenNames">
 *     <span itemprop="givenName">Sarel</span>
 *     <span itemprop="givenName">J.</span>
 *   </span>
 *   <span data-itemprop="familyNames">
 *     <span itemprop="familyName">Fleishman</span>
 *   </span>
 * </li>
 * ```
 */
ready(() => {
  select(':--Person :--name').forEach(span => {
    const name = attr(span, 'content')
    // Aggregate text content of all name elements
    // and then split by spaces
    const givenNames = select(span, ':--givenName')
      .map(item => item.textContent)
      .join(' ')
      .split(/\s+/)
    const familyNames = select(span, ':--familyName')
      .map(item => item.textContent)
      .join(' ')
      .split(/\s+/)

    replace(
      span,
      create(`meta :--name [content=${name}]`),
      create(
        'span :--givenNames',
        ...givenNames.map(givenName => create('span :--givenName', givenName))
      ),
      create(
        'span :--familyNames',
        ...familyNames.map(familyName =>
          create('span :--familyName', familyName)
        )
      )
    )
  })
})
