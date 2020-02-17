/**
 * Functions related to CSS selectors, in particular custome semantic selectors.
 *
 * Designed to be used, as needed, in theme scripts
 * to allow theme authors to use DOM functions like `querySelectorAll`
 * with semantic selectors.
 */

/**
 * Convert a custom semantic selector to the equivalent Microdata
 * property selector.
 *
 * e.g. `:--Article` -> `[itemtype~='http://schema.org/Article']`
 * e.g. `:--author` -> `[itemprop~='http://schema.org/author']`
 *
 * This does the inverse of the mapping defined defined in `./selectors.css`.
 *
 * TODO: Make this smart enough to know which type/props are defined
 * in which content (ie.g. schema.org vs schema.stenci.la etc)
 *
 * TODO: Currently does not deal with `data-itemtype` and `data-itemprop`
 * properties. Not clear if it ever should / can.
 *
 * @param {string} selector
 */
export function semanticToAttributeSelectors(selector: string) {
  return selector.replace(
    /:--(\w+)/g,
    (match, name) =>
      `[${
        /^[A-Z]/.test(name) ? 'itemtype' : 'itemprop'
      }~='http://schema.org/${name}']`
  )
}
