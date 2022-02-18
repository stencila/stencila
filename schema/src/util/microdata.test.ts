/* eslint-disable @typescript-eslint/ban-ts-comment */

import {
  microdataItemtype,
  microdataItemprop,
  microdataType,
  microdata,
  microdataItem,
  microdataProperty,
  microdataRoot,
} from './microdata'
import { codeChunk, article, person, organization } from '../types'

test('microdata', () => {
  // A Stencila type
  expect(microdata(codeChunk({ text: '', programmingLanguage: '' }))).toEqual({
    itemscope: '',
    itemtype: 'https://schema.stenci.la/CodeChunk',
  })

  // A schema.org type
  expect(microdata(article())).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Article',
  })

  // A schema.org type as a Stencila property
  // which is an alias for a schema.org term
  expect(microdata(person(), 'authors')).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Person',
    itemprop: 'author',
  })

  // A schema.org type as a Stencila custom property
  expect(microdata(2, 'depth')).toEqual({
    'itemtype': 'https://schema.org/Number',
    'data-prop': 'depth',
  })

  // Use case: generating a list of authors.
  const authors = [person()]
  // <ol data-prop="authors">
  expect(microdata(authors, 'authors', 'array')).toEqual({
    'data-prop': 'authors',
  })
  // <li itemscope itemtype="https://schema.org/Person" itemprop="author">
  expect(microdata(authors[0], 'authors', 'item')).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Person',
    itemprop: 'author',
  })

  // Use case: generating a list of organizational affiliations
  // for an author where each org is a free-standing element.
  const orgs = [organization()]
  expect(microdata(orgs, 'affiliations', 'array')).toEqual({
    'data-prop': 'affiliations',
  })
  expect(microdata(orgs[0], 'affiliations', 'item', 'org1')).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Organization',
    itemprop: 'affiliation',
    itemref: 'org1',
  })
  expect(microdata(orgs[0], undefined, undefined, 'org1')).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Organization',
    itemid: '#org1',
  })
})

test('microdataItem', () => {
  expect(microdataItem(organization(), 'org1')).toEqual({
    itemscope: '',
    itemtype: 'https://schema.org/Organization',
    itemid: '#org1',
  })
})

test('microdataProperty', () => {
  // A root element for the `affiliations` property e.g. <ol>
  expect(microdataProperty('affiliations', 'array')).toEqual({
    'data-prop': 'affiliations',
  })

  // A child element for an item in the `affiliations` property e.g. <li>
  expect(microdataProperty('affiliations', 'item')).toEqual({
    itemprop: 'affiliation',
  })

  // A child element for an `affiliations` property that is linked to
  // another element
  expect(microdataProperty('affiliations', 'item', 'org1')).toEqual({
    itemprop: 'affiliation',
    itemref: 'org1',
  })

  // A property that is a plural but which aliases another vocab
  // using a different word
  expect(microdataProperty('references', 'item')).toEqual({
    itemprop: 'citation',
  })
})

test('microdataItemtype', () => {
  expect(microdataItemtype('CodeChunk')).toMatch(
    'https://schema.stenci.la/CodeChunk'
  )
  expect(microdataItemtype('Article')).toMatch('https://schema.org/Article')
  // @ts-ignore that Foo is not a type
  expect(microdataItemtype('Foo')).toBeUndefined()
})

test('microdataType', () => {
  expect(microdataType('https://schema.stenci.la/CodeChunk')).toMatch(
    'CodeChunk'
  )
  expect('https://schema.org/Article').toMatch('Article')
  expect(microdataType('https://example.com')).toBeUndefined()
})

test('microdataItemprop', () => {
  expect(microdataItemprop('outputs')).toEqual(['stencila', 'outputs'])

  expect(microdataItemprop('authors')).toEqual(['schema', 'author'])
  expect(microdataItemprop('references')).toEqual(['schema', 'citation'])

  expect(microdataItemprop('foo')).toEqual([undefined, undefined])
})

test('microdataRoot', () => {
  expect(microdataRoot()).toEqual({ 'data-root': '' })
})
