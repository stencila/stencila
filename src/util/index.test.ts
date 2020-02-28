import {
  append,
  before,
  after,
  prepend,
  replace,
  wrap,
  translate,
  create,
  select,
  first,
  ready,
  text,
  attr
} from '.'

const body = document.body

describe('ready', () => {
  it('runs the functions, in order, when the DOM is ready', done => {
    // Monkey patch jsdom to be able to toggle the document's ready state property
    let readyState = 'loading'
    Object.defineProperty(document, 'readyState', {
      get: () => readyState
    })

    body.innerHTML = `<img>`

    ready(() => {
      body.innerHTML += '<br>'
    })

    ready(() => {
      body.innerHTML += '<meta>'
    })

    ready(() => {
      expect(body.innerHTML).toBe('<img><br><meta>')
    })

    // This should trigger a call to all the above
    document.dispatchEvent(new Event('DOMContentLoaded'))
    readyState = 'completed'

    // This should get called straight away and end the test
    ready(done)
  })
})

describe('translate', () => {
  it('works for types', () => {
    expect(translate(':--Article')).toEqual(
      "[itemtype~='http://schema.org/Article']"
    )
    expect(translate(':--CodeChunk')).toEqual(
      "[itemtype~='http://schema.stenci.la/CodeChunk']"
    )
  })

  it('works for properties', () => {
    expect(translate(':--author')).toEqual("[itemprop~='author']")
    expect(translate(':--content')).toEqual("[data-itemprop~='content']")
  })

  it('works for compound selectors', () => {
    expect(translate(':--Article :--author')).toEqual(
      "[itemtype~='http://schema.org/Article'] [itemprop~='author']"
    )
    expect(translate(':--Article > :--author:--Person')).toEqual(
      "[itemtype~='http://schema.org/Article'] > [itemprop~='author'][itemtype~='http://schema.org/Person']"
    )
  })

  it('works for selectors with no custom selectors', () => {
    expect(translate('')).toEqual('')
    expect(translate('.class')).toEqual('.class')
    expect(translate('#id')).toEqual('#id')
    expect(translate('parent > child')).toEqual('parent > child')
  })

  it('throws if it given an unknown custom selector', () => {
    expect(() => translate(':--foo')).toThrow('Unknown custom selector: :--foo')
  })
})

describe('first & select', () => {
  beforeAll(() => {
    body.innerHTML = `
      <div id="div1" itemscope="" itemtype="http://schema.org/Article">
        <span itemprop="author">Jane</span>
        <span itemprop="author">Joe</span>
      </div>
      <div id="div2" itemscope="" itemtype="http://schema.stenci.la/CodeChunk">
        <span itemprop="text">x * y</span>
      </div>
    `
  })

  it('scope is document by default', () => {
    expect(first('div')?.id).toBe('div1')
    expect(select('div').length).toBe(2)
    expect(select('span').length).toEqual(3)
  })

  it('scope is an element if desired', () => {
    expect(first(body, 'span')?.textContent).toBe('Jane')
    expect(select(body, 'span').length).toBe(3)

    expect(first(first('#div2') ?? body, '*')?.textContent).toBe('x * y')
    expect(select(select('#div2')[0], 'span').length).toBe(1)
  })

  it('can use custom selectors', () => {
    expect(first(':--Article')?.id).toBe('div1')
    expect(select(':--Article').length).toBe(1)

    expect(first(':--Article :--author')?.textContent).toBe('Jane')
    expect(select(':--Article :--author').length).toBe(2)

    expect(first(':--CodeChunk :--text')?.textContent).toBe('x * y')
    expect(select(':--CodeChunk :--text')[0].textContent).toBe('x * y')
  })
})

describe('create', () => {
  it('creates an empty div by default', () => {
    const elem = create()

    expect(elem.tagName).toEqual('DIV')
    expect(elem.id).toEqual('')
    expect(elem.className).toEqual('')
    expect(elem.childElementCount).toEqual(0)
  })

  it('works with HTML', () => {
    expect(create('<div>').tagName).toEqual('DIV')

    let elem = create('<span id="foo" class="bar" data-attr="baz">')
    expect(elem.tagName).toEqual('SPAN')
    expect(elem.id).toEqual('foo')
    expect(elem.className).toEqual('bar')
    expect(elem.getAttribute('data-attr')).toEqual('baz')

    elem = create('<ul><li>One</li><li>Two</li></ul>')
    expect(elem.tagName).toEqual('UL')
    expect(elem.children.length).toEqual(2)
  })

  it('works with only the tag name', () => {
    expect(create('div').tagName).toEqual('DIV')
    expect(create('span').tagName).toEqual('SPAN')
  })

  it('works with only id; ignores all but first', () => {
    expect(create('#one').id).toEqual('one')
    expect(create('#one #two').id).toEqual('one')
  })

  it('works with only classes; appends them', () => {
    expect(create('.one').className).toEqual('one')
    expect(create('.one .two').className).toEqual('one two')
  })

  it('works with only attributes', () => {
    expect(create('[one]').getAttribute('one')).toEqual('')
    expect(create('[one="a"]').getAttribute('one')).toEqual('a')
    expect(create('[one="a"] [two="b"]').getAttributeNames()).toEqual([
      'one',
      'two'
    ])
  })

  it('works with semantic selectors', () => {
    expect(create(':--Article').getAttribute('itemscope')).toEqual('')
    expect(create(':--Article').getAttribute('itemtype')).toEqual(
      'http://schema.org/Article'
    )
    expect(create(':--authors').getAttribute('data-itemprop')).toEqual(
      'authors'
    )
    expect(create(':--author').getAttribute('itemprop')).toEqual('author')
  })

  test.each([
    'span #foo .bar [attr="baz"] :--author',
    'span .bar :--author [attr="baz"] #foo',
    'span .bar [itemprop~="author"] [attr="baz"] #foo'
  ])('works with a combination of selectors', selectors => {
    const elem = create(selectors)

    expect(elem.tagName).toEqual('SPAN')
    expect(elem.id).toEqual('foo')
    expect(elem.className).toEqual('bar')
    expect(elem.getAttribute('attr')).toEqual('baz')
    expect(elem.getAttribute('itemprop')).toEqual('author')
  })

  it('creates a clone of provided element', () => {
    const elem1 = create('span#foo.bar[attr="baz"]')
    const elem2 = create(elem1)

    expect(elem1).toEqual(elem2)
  })

  it('can be passed attributes as an object; undefined values ignored', () => {
    const elem = create('span', {
      id: 'foo',
      class: 'bar',
      attr1: 'baz',
      attr2: 42,
      attr3: undefined,
      attr4: true,
      attr5: false
    })

    expect(elem.id).toEqual('foo')
    expect(elem.className).toEqual('bar')
    expect(elem.getAttribute('attr1')).toEqual('baz')
    expect(elem.getAttribute('attr2')).toEqual('42')
    expect(elem.getAttribute('attr3')).toEqual(null)
    expect(elem.getAttribute('attr4')).toEqual('true')
    expect(elem.getAttribute('attr5')).toEqual('false')
  })

  it.each([
    '<span id="quax" class="wix" attr1="xox" attr2="zot">',
    'span #quax .wix [attr1="xox"] [attr2="zot"]'
  ])('attributes as an object override HTML or CSS', spec => {
    const elem = create(spec, { id: 'foo', class: 'bar', attr1: 'baz' })

    expect(elem.id).toEqual('foo')
    expect(elem.className).toEqual('bar')
    expect(elem.getAttribute('attr1')).toEqual('baz')
    expect(elem.getAttribute('attr2')).toEqual('zot')
  })

  it('can be passed child elements', () => {
    const elem = create(
      'div',
      create('span'),
      create('img')
    )
    expect(elem.outerHTML).toEqual('<div><span></span><img></div>')
  })

  it('undefined child elements are ignored', () => {
    const elem = create(
      'div',
      undefined,
      create('span'),
      undefined,
      create('img')
    )
    expect(elem.outerHTML).toEqual('<div><span></span><img></div>')
  })
})

describe('attr', () => {
  it('gets attributes of an element', () => {
    const elem = create('<img foo="bar">')
    expect(attr(elem, 'foo')).toEqual('bar')
    expect(attr(elem, 'baz')).toEqual(undefined)
  })

  it('sets attributes of an element', () => {
    const elem = create('<img>')
    attr(elem, 'foo', 'bar')
    expect(elem.getAttribute('foo')).toEqual('bar')
  })
})

describe('text', () => {
  it('gets the text content of an element', () => {
    expect(text(create('<span>content</span>'))).toEqual('content')
    expect(text(create('<div><span>content</span></div>'))).toEqual('content')
    expect(text(create('<div><span>con</span>tent</div>'))).toEqual('content')
  })

  it('sets the text content', () => {
    let elem = create('<span>content</span>')
    text(elem, 'new')
    expect(text(elem)).toEqual('new')
    expect(elem.outerHTML).toEqual('<span>new</span>')

    elem = create('<div><span>content</span></div>')
    text(elem, 'new')
    expect(text(elem)).toEqual('new')
    expect(elem.outerHTML).toEqual('<div>new</div>')
  })
})

describe('append', () => {
  it('appends elements to a target', () => {
    body.innerHTML = ``

    append(body, create('<img>'))
    expect(body.innerHTML).toEqual('<img>')

    append(body, create('<meta>'))
    expect(body.innerHTML).toEqual('<img><meta>')

    append(body, create('<param>'), create('source'))
    expect(body.innerHTML).toEqual('<img><meta><param><source>')
  })
})

describe('prepend', () => {
  it('prepends elements to a target', () => {
    body.innerHTML = ``

    prepend(body, create('<img>'))
    expect(body.innerHTML).toEqual('<img>')

    prepend(body, create('<meta>'))
    expect(body.innerHTML).toEqual('<meta><img>')

    prepend(body, create('<param>'), create('<source>'))
    expect(body.innerHTML).toEqual('<param><source><meta><img>')
  })
})

describe('before', () => {
  it('inserts elements before an element', () => {
    body.innerHTML = `<div><img></div>`

    before(first('img') ?? body, create('<param>'), create('<source>'))
    expect(body.innerHTML).toEqual('<div><param><source><img></div>')
  })
})

describe('after', () => {
  it('inserts elements after an element', () => {
    body.innerHTML = `<div><img></div>`

    after(first('img') ?? body, create('<param>'), create('<source>'))
    expect(body.innerHTML).toEqual('<div><img><param><source></div>')
  })
})

describe('replace', () => {
  it('replace an element with another', () => {
    body.innerHTML = `<img>`

    replace(first('img') ?? body, create('<param>'))
    expect(body.innerHTML).toEqual('<param>')
  })

  it('replace an element with several others', () => {
    body.innerHTML = `<div><img></div>`

    replace(first('img') ?? body, create('<param>'), create('<source>'))
    expect(body.innerHTML).toEqual('<div><param><source></div>')
  })
})

describe('wrap', () => {
  it('wrap an element with another', () => {
    body.innerHTML = `<img>`

    wrap(first('img') ?? body, create('<div>'))
    expect(body.innerHTML).toEqual('<div><img></div>')
  })
})

test('examples in docs do not error; print outputs', () => {
  const elem = create()

  select(':--CodeChunk')

  select(elem, ':--author')

  const alt1 = create('figure #fig1 .fig :--Figure')
  const alt2 = create('figure', {
    id: 'fig1',
    class: 'fig',
    itemscope: '',
    itemtype: 'http://schema.stenci.la/Figure'
  })
  expect(alt1.outerHTML).toBe(alt2.outerHTML)
  console.log(alt1.outerHTML)

  console.log(create(':--Person', create('span :--name', 'John Doe')).outerHTML)

  text(elem, 'content')
  console.log(text(elem))

  select(':--Figure :--caption').forEach(caption =>
    wrap(caption, create('div'))
  )
})
