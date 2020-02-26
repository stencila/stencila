import {whenReady, select, attr} from '../../util'

test('DOM manipulations', async () => {
  document.body.innerHTML = `
    <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
      <span itemprop="name" content="Sarel J. Fleishman">
        <span itemprop="givenName">Sarel J.</span>
        <span itemprop="familyName">Fleishman</span>
      </span>
    </li>
  `

  await import('.')
  whenReady()

  expect(document.body.innerHTML).toEqual(`
    <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
      <meta itemprop="name" content="Sarel J. Fleishman"><span data-itemprop="givenNames"><span itemprop="givenName">Sarel</span><span itemprop="givenName">J.</span></span><span data-itemprop="familyNames"><span itemprop="familyName">Fleishman</span></span>
    </li>
  `)

  expect(attr(select(':--Person meta:--name')[0], 'content')).toBe(`Sarel J. Fleishman`)
  expect(select(':--Person :--givenNames').length).toBe(1)
  expect(select(':--Person :--givenNames :--givenName').length).toBe(2)
  expect(select(':--Person :--familyNames').length).toBe(1)
  expect(select(':--Person :--familyNames :--familyName').length).toBe(1)

})
