import { Article, Paragraph, Strong, Text } from '@stencila/types'
import * as tmp from 'tmp'
import { test, expect } from 'vitest'

// eslint-disable-next-line import/no-unresolved
import { toPath, toString, fromPath, fromString, fromTo } from './convert.js'

test('fromString', async () => {
  const node = await fromString(
    '{type: "Article", content: [{type: "Paragraph", content: [{type: "Text", value: "Hello world"}]}]}',
    {
      format: 'json5',
    },
  )

  expect(node instanceof Article)
  expect((node as Article).content[0] instanceof Paragraph)
  expect(JSON.stringify(node, null, ' ')).toMatchSnapshot()
})

test('fromPath', async () => {
  const node = await fromPath(
    '../examples/conversion/paragraph/paragraph.json',
  )

  expect(node instanceof Article)
  expect((node as Article).content[0] instanceof Paragraph)
  expect(JSON.stringify(node, null, ' ')).toMatchSnapshot()
})

test('toString', async () => {
  const node = new Article([
    new Paragraph([
      new Text({ string: 'Hello ' }),
      new Strong([new Text({ string: 'again' })]),
      new Text({ string: '!' }),
    ]),
  ])
  const jats = await toString(node, { format: 'jats', compact: false })

  expect(jats).toMatchSnapshot()
})

test('toPath', async () => {
  const original = new Article([
    new Paragraph([new Text({ string: 'Hello file system!' })]),
  ])

  const temp = tmp.fileSync({ postfix: '.jats' }).name
  await toPath(original, temp)
  const roundTrip = await fromPath(temp)
  expect(roundTrip).toEqual(original)
})

test('fromTo', async () => {
  const md = await fromTo(
    '../examples/conversion/paragraph/paragraph.json',
    undefined,
    undefined,
    {
      format: 'md',
    },
  )
  expect(md).toMatchSnapshot()
})
