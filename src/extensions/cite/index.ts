import {
  ready,
  create,
  replace,
  select,
  attr,
  first,
  tag,
  attrs
} from '../../util'

// Import the `person` extension for improved structure of a Person's name
import '../person'

/**
 * This is a temporary patch for https://github.com/stencila/encoda/issues/455
 *
 * Currently, Encoda encodes each `reference` in the `references` property of
 * an article as.
 *
 * ```html
 * <li itemscope="" itemtype="http://schema.org/Article" id="bib42" itemprop="citation">
 *   <div itemscope="" itemtype="http://schema.org/Organization" itemprop="publisher">
 *   ...
 *   </div>
 *   <span data-itemprop="headline">
 *     <meta itemprop="headline" content="A comparative review of short and long neuropeptide F signaling in invertebrates: any similarities to vertebr…">
 *     A comparative review of short and long neuropeptide F signaling in invertebrates: any
 *     similarities to vertebrate neuropeptide Y signaling?
 *   </span>
 *   <meta itemprop="image" content="https://via.placeholder.com/...">
 *   <ol data-itemprop="authors">
 *     <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">...</li>
 *     <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">...</li>
 *   </ol>
 *   <span>
 *     <meta itemprop="datePublished" content="2011">
 *     <time datetime="2011" itemscope="" itemtype="http://schema.org/Date">2011</time>
 *   </span>
 * </li>
 * ```
 *
 * We want to be able to display them using citation styles like APA:
 *
 * ```text
 * Nässel, D. R., & Wegener, C. (2011). A comparative review of short and long
 * neuropeptide F signaling in invertebrates: any similarities to vertebrate
 * neuropeptide Y signaling?. Peptides, 32(6), 1335-1355.
 * ```
 *
 * It would be less work for CSS developers if these properties were in the order most commonly
 * used in citation styles:
 *
 * - authors (as nested spans, rather than an ol)
 * - datePublished
 * - title (aka headline)
 * - isPartOf
 * - publisher
 * - image
 *
 * Note that `publisher` and `image` are always encoded to HTML, even if missing, for conformance with GSDTT,
 * and that some properties are missing in HTML, e.g. `isPartOf` (which provides the `Peptides, 32(6), 1335-1355` bit).
 *
 * Also, currently, `datePublished` is encoded as
 *
 * ```html
 * <span>
 *   <meta itemprop="datePublished" content="2019-08-23">
 *   <time datetime="2019-08-23" itemscope="" itemtype="http://schema.org/Date">2019-08-23</time>
 * </span>
 * ```
 *
 * A better encoding for styling, and concision, is:
 *
 * ```html
 * <time datetime="2019-08-23" itemprop="datePublished" >2019-08-23</time>
 * ```
 *
 * Noting that for `<time>` elements the `datetime` attribute is used as the property value:
 * https://www.w3.org/TR/microdata/#values and it should not have `itemscope` and `itemtype`
 * attributes.
 */
ready(() =>
  select(':--references :--reference').forEach(reference => {
    // Change `authors` property from list to nested spans
    select(reference, 'ol:--authors').forEach(authors => {
      select(authors, 'li:--author').forEach(author =>
        replace(author, tag(author, 'span'))
      )
      return replace(authors, tag(authors, 'span'))
    })

    // If `datePublished` is inside a span then un-wrap it
    select(reference, 'span > :--datePublished').forEach(elem => {
      const date = attr(elem, 'content')
      replace(
        elem.parentElement as Element,
        create(`time [datetime=${date}] :--datePublished`, date)
      )
    })

    // If `publisher` is a div make it a span
    select(reference, 'div:--publisher').forEach(elem =>
      replace(elem, tag(elem, 'span'))
    )

    replace(
      reference,
      create(
        tag(reference),
        attrs(reference),
        first(':--authors'),
        first(':--datePublished'),
        first(':--title'),
        first(':--publisher'),
        first(':--image')
      )
    )
  })
)
