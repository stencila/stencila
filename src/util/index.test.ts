import { create } from '.'

describe('create', () => {
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

  it('works with only id', () => {
    expect(create('#one').id).toEqual('one')
    expect(create('#one').id).toEqual('one')
  })

  it('works with only classes', () => {
    expect(create('.one').className).toEqual('one')
    expect(create('.one .two').className).toEqual('one two')
  })

  it('works with only attributes', () => {
    expect(create('[one]').getAttribute('one')).toEqual('')
    expect(create('[one="a"]').getAttribute('one')).toEqual('a')
    expect(create('[one="a"] [two=\'b\']').getAttributeNames()).toEqual([
      'one',
      'two'
    ])
  })

  it('works with a combination of selectors', () => {
    const elem = create('span#foo.bar[attr="baz"]')

    expect(elem.tagName).toEqual('SPAN')
    expect(elem.id).toEqual('foo')
    expect(elem.className).toEqual('bar')
    expect(elem.getAttribute('attr')).toEqual('baz')
  })
})
