import { select, text, whenReady } from '../../util'

const body = document.body

test('DOM manipulations', async () => {
  body.innerHTML = `
  <section data-itemprop="references">
    <li itemscope="" itemtype="http://schema.org/Person" itemprop="author">
       <span itemprop="name" content="Sarel J. Fleishman">
         <span itemprop="givenName">Sarel J.</span>
         <span itemprop="familyName">Fleishman</span>
       </span>
    </li>
  </section>
  `

  await import('.')
  whenReady()

  expect(select(':--givenName').map(elem => text(elem))).toEqual(['S', 'J'])
})
