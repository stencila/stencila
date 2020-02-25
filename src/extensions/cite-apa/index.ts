import '../cite'
import { ready, select } from '../../util'

/**
 * APA style uses author initials (rather than complete given names).
 * This is difficult to achieve using CSS alone, so truncate given names here.
 */
ready(() =>
  select(':--references :--givenName').forEach(elem => {
    elem.innerHTML = elem.textContent?.[0] ?? ''
  })
)
