/* eslint-disable @typescript-eslint/ban-ts-ignore */

import {
  microdataItemtype,
  microdataItemprop,
  microdataType,
  microdata,
  microdataItem,
  microdataProperty
} from './microdata'
import { codeChunk, article, person, thing, organization } from '../types'

test('microdata', () => {
  // A Stencila type
  expect(microdata(codeChunk({ text: '' }))).toEqual({
    itemscope: '',
    itemtype: 'http://schema.stenci.la/CodeChunk'
  })

  // A schema.org type
  expect(microdata(article())).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Article'
  })

  // A schema.org type as a Stencila property
  // which is an alias for a schema.org term
  expect(microdata(person(), 'authors')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Person',
    itemprop: 'author'
  })

  // A schema.org type as a Stencila custom property
  expect(microdata(2, 'depth')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Number',
    'data-itemprop': 'depth'
  })

  // Generate microdata for a property who's node
  // is encoded as a free-standing element. Use id
  // for this.
  const org1 = organization()
  expect(microdata(org1, 'affiliations', 'org1')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Organization',
    itemprop: 'affiliation',
    itemref: 'org1'
  })
  expect(microdata(org1, undefined, 'org1')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Organization',
    itemid: '#org1'
  })
})

test('microdataItem', () => {
  expect(microdataItem(organization(), 'org1')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Organization',
    itemid: '#org1'
  })
})

test('microdataProperty', () => {
  expect(microdataProperty('affiliations', 'org1')).toEqual({
    itemprop: 'affiliation',
    itemref: 'org1'
  })
})

test('microdataItemtype', () => {
  expect(microdataItemtype('CodeChunk')).toMatch(
    'http://schema.stenci.la/CodeChunk'
  )
  expect(microdataItemtype('Article')).toMatch('http://schema.org/Article')
  // @ts-ignore that Foo is not a type
  expect(microdataItemtype('Foo')).toBeUndefined()
})

test('microdataType', () => {
  expect(microdataType('http://schema.stenci.la/CodeChunk')).toMatch(
    'CodeChunk'
  )
  expect('http://schema.org/Article').toMatch('Article')
  expect(microdataType('http://example.com')).toBeUndefined()
})

test('microdataItemprop', () => {
  expect(microdataItemprop('outputs')).toEqual(['stencila', 'outputs'])

  expect(microdataItemprop('authors')).toEqual(['schema', 'author'])
  expect(microdataItemprop('references')).toEqual(['schema', 'citation'])

  expect(microdataItemprop('maintainers')).toEqual(['codemeta', 'maintainer'])

  expect(microdataItemprop('foo')).toEqual([undefined, undefined])
})
