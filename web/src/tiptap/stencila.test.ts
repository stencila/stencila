/**
 * Unit tests for Stencila-specific Tiptap extensions and their JSON shape.
 */
import { Editor, type JSONContent } from '@tiptap/core'
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

describe('Stencila Tiptap extensions', () => {
  it('serializes supported and opaque nodes in canonical JSON shape', () => {
    const editor = createEditor({
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
    })

    try {
      expect(JSON.stringify(editor.getJSON())).toBe(
        `{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]},{"type":"paragraph","content":[{"type":"text","text":"Hello "},{"type":"text","marks":[{"type":"bold"}],"text":"bold"},{"type":"text","text":" and "},{"type":"text","marks":[{"type":"italic"}],"text":"italic"},{"type":"text","text":" "},{"type":"stencilaInline","attrs":{"nodeType":"MathInline","node":{"type":"MathInline","code":"x + y"}}}]},{"type":"stencilaBlock","attrs":{"nodeType":"CodeChunk","node":{"type":"CodeChunk","code":"print('hello')"}}}]}`
      )
    } finally {
      editor.destroy()
    }
  })
})
