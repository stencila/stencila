/* eslint-disable @typescript-eslint/ban-ts-ignore */

import {
  microdataItemtype,
  microdataItemprop,
  microdataType,
  microdata,
  microdataItem,
  microdataProperty,
  microdataRoot
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

  // Use case: generating a list of authors.
  const authors = [person()]
  // <ol data-itemprop="authors">
  expect(microdata(authors, 'authors', 'array')).toEqual({
    'data-itemprop': 'authors'
  })
  // <li itemscope itemtype="http://schema.org/Person" itemprop="author">
  expect(microdata(authors[0], 'authors', 'item')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Person',
    itemprop: 'author'
  })

  // Use case: generating a list of organizational affiliations
  // for an author where each org is a free-standing element.
  const orgs = [organization()]
  expect(microdata(orgs, 'affiliations', 'array')).toEqual({
    'data-itemprop': 'affiliations'
  })
  expect(microdata(orgs[0], 'affiliations', 'item', 'org1')).toEqual({
    itemscope: '',
    itemtype: 'http://schema.org/Organization',
    itemprop: 'affiliation',
    itemref: 'org1'
  })
  expect(microdata(orgs[0], undefined, undefined, 'org1')).toEqual({
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
  // A root element for the `affiliations` property e.g. <ol>
  expect(microdataProperty('affiliations', 'array')).toEqual({
    'data-itemprop': 'affiliations'
  })

  // A child element for an item in the `affiliations` property e.g. <li>
  expect(microdataProperty('affiliations', 'item')).toEqual({
    itemprop: 'affiliation'
  })

  // A child element for an `affiliations` property that is linked to
  // another element
  expect(microdataProperty('affiliations', 'item', 'org1')).toEqual({
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

test('microdataRoot', () => {
  expect(microdataRoot()).toEqual({'data-itemscope': 'root'})
})
