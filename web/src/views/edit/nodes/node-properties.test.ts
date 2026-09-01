import { Editor, type JSONContent } from '@tiptap/core'
import { NodeSelection, TextSelection } from '@tiptap/pm/state'
import { describe, expect, it } from 'vitest'

import { createStencilaTiptapExtensions } from '../../../tiptap/extensions'

import {
  findEditNodePropertyTarget,
  listResearchObjectTargets,
  normalizePersistentIdInput,
  relationKindRange,
  relationKindsForSource,
  setEditNodePropertiesTransaction,
  validatePersistentIdInput,
  validatePersistentIdRemoval,
  validateRelationDrafts,
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

function dispatchMathProperties(
  editor: Editor,
  mathLanguage: string | null
) {
  const target = findEditNodePropertyTarget(editor.state)
  if (!target) {
    throw new Error('Expected editable node property target')
  }

  const transaction = setEditNodePropertiesTransaction(editor.state, target, {
    mathLanguage,
  })
  if (!transaction) {
    throw new Error('Expected math properties transaction')
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

  it('updates math block ids and math language attrs', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'mathBlock',
          attrs: {
            code: 'x + y',
            mathLanguage: 'tex',
            mathml: '<math><mi>x</mi></math>',
            images: [{ contentUrl: 'old.png' }],
            compilationMessages: [{ level: 'Warning' }],
          },
        },
      ],
    })

    try {
      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.typeName).toBe('mathBlock')
      expect(target?.displayName).toBe('Math Block')
      expect(target?.mathLanguage).toBe('tex')

      dispatchPersistentId(editor, 'eq-1')
      dispatchMathProperties(editor, 'asciimath')

      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'mathBlock',
        attrs: {
          id: 'eq-1',
          code: 'x + y',
          mathLanguage: 'asciimath',
          mathml: null,
          images: null,
          compilationMessages: null,
        },
      })
    } finally {
      editor.destroy()
    }
  })

  it('updates inline math properties when selected directly', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'Let ' },
            {
              type: 'mathInline',
              attrs: {
                code: 'x',
                mathLanguage: 'tex',
              },
            },
          ],
        },
      ],
    })

    try {
      let mathPos: number | undefined
      editor.state.doc.descendants((node, pos) => {
        if (node.type.name === 'mathInline') {
          mathPos = pos
          return false
        }

        return true
      })

      if (mathPos === undefined) {
        throw new Error('Expected inline math position')
      }

      editor.view.dispatch(
        editor.state.tr.setSelection(NodeSelection.create(editor.state.doc, mathPos))
      )

      const target = findEditNodePropertyTarget(editor.state)
      expect(target?.typeName).toBe('mathInline')
      expect(target?.mathLanguage).toBe('tex')

      dispatchMathProperties(editor, null)

      const paragraph = editor.getJSON().content?.[0]
      expect(paragraph?.content?.[1]).toMatchObject({
        type: 'mathInline',
        attrs: {
          code: 'x',
          mathLanguage: null,
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

function researchObject(
  type: string,
  attrs: Record<string, unknown>,
  text: string
): JSONContent {
  return {
    type,
    attrs,
    content: [
      {
        type: 'paragraph',
        content: [{ type: 'text', text }],
      },
    ],
  }
}

describe('ResearchObject properties and relations', () => {
  it('targets a wrapper from its editable content', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        researchObject(
          'claim',
          {
            id: 'claim-1',
            label: 'Claim 1',
            title: 'Main claim',
            claimType: 'Hypothesis',
          },
          'A claim'
        ),
      ],
    })

    try {
      selectText(editor, 'A claim')
      expect(findEditNodePropertyTarget(editor.state)).toMatchObject({
        typeName: 'claim',
        persistentId: 'claim-1',
        researchObjectLabel: 'Claim 1',
        researchObjectTitle: 'Main claim',
        claimType: 'Hypothesis',
      })
    } finally {
      editor.destroy()
    }
  })

  it('orders relation choices by advisory MIRA semantics', () => {
    expect(relationKindsForSource('question')).toEqual([
      'AddressedBy',
      'Supports',
      'SupportedBy',
      'Opposes',
      'OpposedBy',
      'Addresses',
      'Follows',
      'Grounds',
      'IsGroundedIn',
      'RequestFor',
      'RequestTarget',
    ])
    expect(relationKindRange('SupportedBy')).toEqual(['claim', 'evidence'])
  })

  it('lists every other ResearchObject as a potential target', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        researchObject('claim', { id: 'claim-1' }, 'A claim'),
        researchObject(
          'evidence',
          { id: 'evidence-1', label: 'Evidence 1' },
          'Evidence'
        ),
        researchObject('protocol', {}, 'Protocol'),
      ],
    })

    try {
      selectText(editor, 'A claim')
      const source = findEditNodePropertyTarget(editor.state)
      if (!source) {
        throw new Error('Expected ResearchObject source')
      }
      expect(listResearchObjectTargets(editor.state, source.pos)).toMatchObject([
        {
          typeName: 'evidence',
          id: 'evidence-1',
          label: 'Evidence 1',
        },
        { typeName: 'protocol', excerpt: 'Protocol' },
      ])
    } finally {
      editor.destroy()
    }
  })

  it('assigns an ID to an id-less in-document relation target', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        researchObject('claim', { id: 'claim-1' }, 'A claim'),
        researchObject('evidence', {}, 'Evidence'),
      ],
    })

    try {
      selectText(editor, 'A claim')
      const source = findEditNodePropertyTarget(editor.state)
      if (!source) {
        throw new Error('Expected ResearchObject source')
      }
      const [evidence] = listResearchObjectTargets(editor.state, source.pos)
      const transaction = setEditNodePropertiesTransaction(
        editor.state,
        source,
        {
          relations: [
            {
              kind: 'SupportedBy',
              target: '',
              targetPos: evidence.pos,
            },
          ],
        }
      )
      if (!transaction) {
        throw new Error('Expected relation transaction')
      }
      editor.view.dispatch(transaction)

      const content = editor.getJSON().content ?? []
      const evidenceId = content[1]?.attrs?.id as string
      expect(evidenceId).toMatch(/^evidence-[0-9a-f]{8}$/)
      expect(content[0]?.attrs?.relations).toEqual([
        { kind: 'SupportedBy', target: `#${evidenceId}` },
      ])
    } finally {
      editor.destroy()
    }
  })

  it('rewrites incoming relations when a target ID changes', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        researchObject('claim', { id: 'claim-old' }, 'A claim'),
        researchObject(
          'evidence',
          { relations: [{ kind: 'Supports', target: '#claim-old' }] },
          'Evidence'
        ),
        researchObject(
          'request',
          { relations: [{ kind: 'RequestTarget', target: 'claim-old' }] },
          'Request'
        ),
      ],
    })

    try {
      selectText(editor, 'A claim')
      dispatchPersistentId(editor, 'claim-new')

      const content = editor.getJSON().content ?? []
      expect(content[0]?.attrs?.id).toBe('claim-new')
      expect(content[1]?.attrs?.relations?.[0]?.target).toBe('#claim-new')
      expect(content[2]?.attrs?.relations?.[0]?.target).toBe('#claim-new')
    } finally {
      editor.destroy()
    }
  })

  it('blocks removal of a referenced ResearchObject ID', () => {
    const editor = createEditor({
      type: 'doc',
      content: [
        researchObject('claim', { id: 'claim-1' }, 'A claim'),
        researchObject(
          'evidence',
          { relations: [{ kind: 'Supports', target: '#claim-1' }] },
          'Evidence'
        ),
      ],
    })

    try {
      selectText(editor, 'A claim')
      const target = findEditNodePropertyTarget(editor.state)
      if (!target) {
        throw new Error('Expected ResearchObject target')
      }
      expect(validatePersistentIdRemoval(editor.state, target)).toEqual({
        ok: false,
        message: 'Persistent id is referenced by 1 relation',
      })
      expect(
        setEditNodePropertiesTransaction(editor.state, target, {
          persistentId: null,
        })
      ).toBeUndefined()
    } finally {
      editor.destroy()
    }
  })

  it('accepts syntactically valid non-recommended relations', () => {
    expect(validateRelationDrafts([{ kind: 'Follows', target: '#claim-1' }]))
      .toEqual({ ok: true })
    expect(validateRelationDrafts([{ kind: 'Follows', target: 'two words' }]))
      .toEqual({
        ok: false,
        message: 'Relation target cannot contain spaces',
      })
  })
})
