import { attr, first, whenReady } from '../../util/index'

test('Removal of inner itemscope and itemtype from CodeBlocks', async () => {
  document.body.innerHTML = `
    <pre itemscope="" itemtype="http://schema.stenci.la/CodeBlock">
      <code itemscope="" itemtype="http://schema.stenci.la/CodeFragment">
      </code>
    </pre>
  `
  await import('.')
  whenReady()

  const code = first('code') ?? document.body
  expect(code.tagName).toBe('CODE')
  expect(attr(code, 'itemscope')).toBe(null)
  expect(attr(code, 'itemtype')).toBe(null)
  expect(attr(code, 'class')).toBe('language-text')
})
