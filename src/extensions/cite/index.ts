import { ready, create, replace, select } from '../../util'

/**
 * Currently, Encoda encodes a `CreativeWork.datePublished` as
 *
 * ```html
 * <span>
 *   <meta itemprop="datePublished" content="2019-08-23">
 *   <time datetime="2019-08-23" itemscope="" itemtype="http://schema.org/Date">2019-08-23</time>
 * </span>
 * ```
 *
 * So that we can style the date, a better encoding would simply be:
 *
 * ```html
 * <time datetime="2019-08-23" itemscope="" itemtype="http://schema.org/Date" itemprop="datePublished" >2019-08-23</time>
 * ```
 *
 * Noting that for `<time>` elements the `datetime` attribute is used as the property value:
 * https://www.w3.org/TR/microdata/#values
 */
ready(() =>
  select('span > meta[itemprop="datePublished"]').forEach(meta => {
    const date = meta.getAttribute('content')
    replace(
      meta.parentElement as Element,
      create(
        `time[datetime=${date}][itemscope][itemtype="http://schema.org/Date"][itemprop=datePublished]`,
        date as string
      )
    )
  })
)
