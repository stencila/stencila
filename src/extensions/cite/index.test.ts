import { whenReady, select, tag, attr } from '../../util'

test('DOM manipulations', async () => {
  const body = document.body
  body.innerHTML = `
<section data-itemprop="references">
  <ol>
    <li itemscope="" itemtype="http://schema.org/Article" id="bib1" itemprop="citation">
      <div itemscope="" itemtype="http://schema.org/Organization" itemprop="publisher">
        <meta itemprop="name" content="Unknown">
        <div itemscope="" itemtype="http://schema.org/ImageObject" itemprop="logo">
          <meta itemprop="url" content="https://via.placeholder.com/" src="https://via.placeholder.com/">
        </div>
      </div><span itemprop="headline">The continuing case for the Renshaw cell</span>
      <meta itemprop="image" content="https://via.placeholder.com/">
      <ol data-itemprop="authors">
        <li itemscope="" itemtype="http://schema.org/Person" itemprop="author"><span itemprop="name" content="FJ Alvarez"><span itemprop="givenName">FJ</span><span itemprop="familyName">Alvarez</span></span></li>
        <li itemscope="" itemtype="http://schema.org/Person" itemprop="author"><span itemprop="name" content="REW Fyffe"><span itemprop="givenName">REW</span><span itemprop="familyName">Fyffe</span></span></li>
      </ol><span>
        <meta itemprop="datePublished" content="2007"><time datetime="2007" itemscope="" itemtype="http://schema.org/Date">2007</time></span>
    </li>
  </ol>
</section>
  `

  await import('.')
  whenReady()

  expect(select('ol:--authors').length).toBe(0)
  expect(select('span:--authors').length).toBe(1)

  expect(select('li:--author').length).toBe(0)
  expect(select('span:--author').length).toBe(2)

  expect(
    select(':--reference > *').map(elem => [
      tag(elem).toLowerCase(),
      attr(elem, 'itemprop') ?? attr(elem, 'data-itemprop')
    ])
  ).toEqual([
    ['span', 'authors'],
    ['time', 'datePublished'],
    ['span', 'headline'],
    ['span', 'publisher'],
    ['meta', 'image']
  ])
})
