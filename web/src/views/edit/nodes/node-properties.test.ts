import { Editor, type JSONContent } from '@tiptap/core'
import { NodeSelection, TextSelection } from '@tiptap/pm/state'
import { describe, expect, it } from 'vitest'

import { createStencilaTiptapExtensions } from '../../../tiptap/extensions'

import {
  findEditNodePropertyTarget,
  normalizePersistentIdInput,
  setEditNodePropertiesTransaction,
  validatePersistentIdInput,
} from './node-properties'

function createEditor(content: JSONContent): Editor {
  return new Editor({
    element: null,
    extensions: createStencilaTiptapExtensions(),
    content,
  })
}

function dispatchPersistentId(editor: Editor, value: string | null) {
  const target = findEditNodePropertyTarget(editor.state)
  if (!target) {
    throw new Error('Expected editable node property target')
  }

  const transaction = setEditNodePropertiesTransaction(editor.state, target, {
    persistentId: value,
  })
  if (!transaction) {
    throw new Error('Expected persistent id transaction')
  }

  editor.view.dispatch(transaction)
}

function dispatchCodeBlockProperties(
  editor: Editor,
  programmingLanguage: string | null,
  isDemo: boolean | null
) {
  const target = findEditNodePropertyTarget(editor.state)
  if (!target) {
    throw new Error('Expected editable node property target')
  }

  const transaction = setEditNodePropertiesTransaction(editor.state, target, {
    programmingLanguage,
    isDemo,
  })
  if (!transaction) {
    throw new Error('Expected code block properties transaction')
  }

  editor.view.dispatch(transaction)
}

function selectText(editor: Editor, text: string) {
  let textPos: number | undefined

  editor.state.doc.descendants((node, pos) => {
    if (node.isText && node.text === text) {
      textPos = pos
      return false
    }

    return true
  })

  if (textPos === undefined) {
    throw new Error(`Expected text node: ${text}`)
  }

  editor.view.dispatch(
    editor.state.tr.setSelection(TextSelection.create(editor.state.doc, textPos))
  )
}

describe('edit node property helpers', () => {
  it('normalizes pasted hash ids', () => {
    expect(normalizePersistentIdInput(' #setup-code ')).toBe('setup-code')
  })

  it('uses the programming language as the summary label for code nodes', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            language: 'python',
          },
          content: [{ type: 'text', text: 'print("hello")' }],
        },
      ],
    })

    try {
      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.displayName).toBe('Code Block')
      expect(target?.summaryLabel).toBe('Python')
      expect(target?.persistentId).toBeUndefined()
    } finally {
      editor.destroy()
    }
  })

  it('falls back to the type name when a code node has no language', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          content: [{ type: 'text', text: 'print("hello")' }],
        },
      ],
    })

    try {
      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.summaryLabel).toBe('Code Block')
    } finally {
      editor.destroy()
    }
  })

  it('adds and removes persistent ids on code blocks', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            language: 'python',
          },
          content: [{ type: 'text', text: 'print("hello")' }],
        },
      ],
    })

    try {
      dispatchPersistentId(editor, 'setup-code')

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'codeBlock',
        attrs: {
          id: 'setup-code',
          language: 'python',
        },
      })

      dispatchPersistentId(editor, null)

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'codeBlock',
        attrs: {
          id: null,
          language: 'python',
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('changes code block programming language and demo properties', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            language: 'python',
          },
          content: [{ type: 'text', text: 'print("hello")' }],
        },
      ],
    })

    try {
      dispatchCodeBlockProperties(editor, 'javascript', true)

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'codeBlock',
        attrs: {
          language: 'javascript',
          isDemo: true,
        },
      })

      dispatchCodeBlockProperties(editor, null, null)

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'codeBlock',
        attrs: {
          language: null,
          isDemo: null,
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('targets table properties from inside a table cell', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'table',
          content: [
            {
              type: 'tableRow',
              content: [
                {
                  type: 'tableCell',
                  attrs: {
                    align: null,
                    colspan: 1,
                    rowspan: 1,
                    colwidth: null,
                  },
                  content: [
                    {
                      type: 'paragraph',
                      content: [{ type: 'text', text: 'Data' }],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    })

    try {
      selectText(editor, 'Data')

      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.typeName).toBe('table')

      dispatchPersistentId(editor, 'results-table')

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'table',
        attrs: {
          id: 'results-table',
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('updates persistent ids inside opaque Stencila block payloads', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'stencilaBlock',
          attrs: {
            nodeType: 'Figure',
            node: {
              type: 'Figure',
              label: '1',
            },
          },
        },
      ],
    })

    try {
      editor.view.dispatch(
        editor.state.tr.setSelection(NodeSelection.create(editor.state.doc, 0))
      )

      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.displayName).toBe('Figure')
      expect(target?.summaryLabel).toBe('Figure 1')

      dispatchPersistentId(editor, 'figure-1')

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'stencilaBlock',
        attrs: {
          nodeType: 'Figure',
          node: {
            type: 'Figure',
            id: 'figure-1',
            label: '1',
          },
        },
      })

      dispatchPersistentId(editor, null)

      const block = editor.getJSON().content?.[0]
      expect(block?.attrs?.node).not.toHaveProperty('id')
    } finally {
      editor.destroy()
    }
  })

  it('rejects duplicate persistent ids', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            id: 'setup-code',
          },
          content: [{ type: 'text', text: 'a = 1' }],
        },
        {
          type: 'codeBlock',
          attrs: {
            id: 'analysis-code',
          },
          content: [{ type: 'text', text: 'b = 2' }],
        },
      ],
    })

    try {
      const target = findEditNodePropertyTarget(editor.state)
      if (!target) {
        throw new Error('Expected editable node property target')
      }

      expect(
        validatePersistentIdInput('analysis-code', editor.state, target.pos)
      ).toEqual({
        ok: false,
        message: 'Persistent id already exists',
      })

      expect(validatePersistentIdInput('#setup-code', editor.state, target.pos))
        .toEqual({
          ok: true,
          value: 'setup-code',
        })
    } finally {
      editor.destroy()
    }
  })

  it('rejects duplicate persistent ids in opaque inline payloads', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: {
            id: 'setup-code',
          },
          content: [{ type: 'text', text: 'a = 1' }],
        },
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'See ' },
            {
              type: 'stencilaInline',
              attrs: {
                nodeType: 'MathInline',
                node: {
                  type: 'MathInline',
                  id: 'equation-1',
                  code: 'x + y',
                },
              },
            },
          ],
        },
      ],
    })

    try {
      const target = findEditNodePropertyTarget(editor.state)
      if (!target) {
        throw new Error('Expected editable node property target')
      }

      expect(validatePersistentIdInput('equation-1', editor.state, target.pos))
        .toEqual({
          ok: false,
          message: 'Persistent id already exists',
        })
    } finally {
      editor.destroy()
    }
  })
})
