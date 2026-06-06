/**
 * Unit tests for Stencila-specific Tiptap extensions and their JSON shape.
 */
import { Editor, type JSONContent, getSchema } from '@tiptap/core'
import { describe, expect, it } from 'vitest'

import { createStencilaTiptapExtensions } from './extensions'

/**
 * Create a Tiptap editor using the same core extensions as the edit view.
 */
function createEditor(content: JSONContent): Editor {
  return new Editor({
    element: null,
    extensions: createStencilaTiptapExtensions(),
    content,
  })
}

/**
 * Supported native marks plus opaque Stencila extension nodes.
 */
function supportedTiptapJson(): JSONContent {
  return {
    type: 'doc',
    content: [
      {
        type: 'heading',
        attrs: { level: 2 },
        content: [{ type: 'text', text: 'Title' }],
      },
      {
        type: 'paragraph',
        content: [
          { type: 'text', text: 'Hello ' },
          {
            type: 'text',
            marks: [{ type: 'bold' }],
            text: 'bold',
          },
          { type: 'text', text: ' and ' },
          {
            type: 'text',
            marks: [{ type: 'italic' }],
            text: 'italic',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [
              {
                type: 'link',
                attrs: {
                  href: 'https://example.com',
                  title: 'Example',
                  rel: 'noopener',
                  labelOnly: true,
                },
              },
            ],
            text: 'link',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [
              {
                type: 'code',
                attrs: {
                  programmingLanguage: 'typescript',
                },
              },
            ],
            text: 'code',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'strike' }],
            text: 'strike',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'underline' }],
            text: 'under',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'subscript' }],
            text: 'sub',
          },
          { type: 'text', text: ' ' },
          {
            type: 'text',
            marks: [{ type: 'superscript' }],
            text: 'sup',
          },
          { type: 'text', text: ' ' },
          {
            type: 'stencilaInline',
            attrs: {
              nodeType: 'MathInline',
              node: { type: 'MathInline', code: 'x + y' },
            },
          },
        ],
      },
      {
        type: 'stencilaBlock',
        attrs: {
          nodeType: 'CodeChunk',
          node: { type: 'CodeChunk', code: "print('hello')" },
        },
      },
    ],
  }
}

describe('Stencila Tiptap extensions', () => {
  it('serializes supported and opaque nodes in canonical JSON shape', () => {
    const editor = createEditor(supportedTiptapJson())

    try {
      expect(editor.getJSON()).toEqual(supportedTiptapJson())
    } finally {
      editor.destroy()
    }
  })

  it('defines link marks as anchor elements', () => {
    const schema = getSchema(createStencilaTiptapExtensions())
    const render = schema.marks.link.spec.toDOM
    const mark = schema.marks.link.create({
      href: 'https://example.com',
      title: 'Example',
      rel: 'noopener',
      labelOnly: true,
    })

    expect(render?.(mark, true)).toEqual([
      'a',
      {
        href: 'https://example.com',
        title: 'Example',
        rel: 'noopener',
      },
      0,
    ])
  })

  it('registers the link click handler', () => {
    const editor = createEditor({
      type: 'doc',
      content: [],
    })

    try {
      expect(
        editor.extensionManager.plugins.some((plugin) =>
          Boolean(plugin.props.handleClick)
        )
      ).toBe(true)
    } finally {
      editor.destroy()
    }
  })
})
