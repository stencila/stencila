import { create, first, ready, select, wrap } from '../../util'

ready(() => {
  // Add wrappers around elements in the abstract to be able to use
  // Bootstrap's `panel` class
  select(':--Article :--description').forEach(desc => {
    const h2 = first(desc, 'h2')
    if (h2 !== null) wrap(h2, create('div .panel-heading'))
    const p = first(desc, 'p')
    if (p !== null) wrap(p, create('div .panel-body'))
  })
})
