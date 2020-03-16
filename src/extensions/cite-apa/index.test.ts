import { select, text, whenReady } from '../../util'

const body = document.body

test('DOM manipulations', async () => {
  body.innerHTML = `
<section data-itemprop="references">
  <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
    <meta itemprop="name" content="Sariel J Fleishman">
    <span data-itemprop="givenNames">
      <span itemprop="givenName">Sariel</span>
      <span itemprop="givenName">J</span>
    </span>
    <span data-itemprop="familyNames">
      <span itemprop="familyName">Fleishman</span>
    </span>
  </li>
</section>
  `

  await import('.')
  whenReady()

  expect(select(':--givenName').map(elem => text(elem))).toEqual(['S', 'J'])
})
