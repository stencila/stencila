import { Editor, type JSONContent, getSchema } from '@tiptap/core'
import { TextSelection } from '@tiptap/pm/state'
import { describe, expect, it } from 'vitest'

import { createStencilaTiptapExtensions } from './extensions'
import {
  RESEARCH_OBJECT_NODE_NAMES,
  activeResearchObject,
  canWrapInResearchObject,
  recommendedRelationKinds,
  recommendedRelationTargets,
  unwrapResearchObject,
  wrapInResearchObject,
} from './research-objects'
import { filterSlashMenuItems } from './slash-menu'

function createEditor(content: JSONContent): Editor {
  return new Editor({
    element: null,
    extensions: createStencilaTiptapExtensions(),
    content,
  })
}

function paragraphs(): JSONContent {
  return {
    type: 'doc',
    content: [
      { type: 'paragraph', content: [{ type: 'text', text: 'one' }] },
      { type: 'paragraph', content: [{ type: 'text', text: 'two' }] },
    ],
  }
}

function selectAcrossBlocks(editor: Editor) {
  editor.view.dispatch(
    editor.state.tr.setSelection(
      TextSelection.create(editor.state.doc, 2, editor.state.doc.content.size - 2)
    )
  )
}

describe('ResearchObject Tiptap module', () => {
  it('registers every native ResearchObject node', () => {
    const schema = getSchema(createStencilaTiptapExtensions())
    for (const name of RESEARCH_OBJECT_NODE_NAMES) {
      expect(schema.nodes[name], name).toBeDefined()
    }
  })

  it('wraps and unwraps a multi-block selection', () => {
    const editor = createEditor(paragraphs())
    try {
      selectAcrossBlocks(editor)
      expect(canWrapInResearchObject(editor)).toBe(true)
      expect(wrapInResearchObject(editor, 'claim')).toBe(true)
      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'claim',
        content: paragraphs().content,
      })
      expect(activeResearchObject(editor)).toBe('claim')
      expect(unwrapResearchObject(editor)).toBe(true)
      expect(editor.getJSON()).toEqual(paragraphs())
    } finally {
      editor.destroy()
    }
  })

  it('supports nested ResearchObject wrappers', () => {
    const editor = createEditor(paragraphs())
    try {
      selectAcrossBlocks(editor)
      expect(wrapInResearchObject(editor, 'claim')).toBe(true)
      expect(wrapInResearchObject(editor, 'evidence')).toBe(true)
      expect(editor.getJSON().content?.[0]).toMatchObject({
        type: 'claim',
        content: [{ type: 'evidence', content: paragraphs().content }],
      })
      expect(activeResearchObject(editor)).toBe('evidence')
      expect(unwrapResearchObject(editor)).toBe(true)
      expect(activeResearchObject(editor)).toBe('claim')
      expect(unwrapResearchObject(editor)).toBe(true)
      expect(editor.getJSON()).toEqual(paragraphs())
    } finally {
      editor.destroy()
    }
  })

  it('preserves card metadata attributes through editor JSON', () => {
    const content: JSONContent = {
      type: 'doc',
      content: [
        {
          type: 'claim',
          attrs: {
            id: 'claim-1',
            relations: [{ kind: 'SupportedBy', target: '#evidence-1' }],
            metadata: { source: 'authored' },
            label: 'Claim 1',
            title: 'Main claim',
            claimType: 'Hypothesis',
          },
          content: [
            {
              type: 'paragraph',
              content: [{ type: 'text', text: 'A claim' }],
            },
          ],
        },
      ],
    }
    const editor = createEditor(content)
    try {
      expect(editor.getJSON()).toEqual(content)
    } finally {
      editor.destroy()
    }
  })

  it('keeps MIRA relation semantics advisory', () => {
    expect(recommendedRelationKinds('claim')).toEqual([
      'Supports',
      'SupportedBy',
      'Opposes',
      'OpposedBy',
      'Addresses',
    ])
    expect(recommendedRelationTargets('SupportedBy')).toEqual([
      'claim',
      'evidence',
    ])
    expect(recommendedRelationTargets('RequestFor')).toEqual([])
  })
})

describe('ResearchObject slash menu', () => {
  it('offers and filters wrap actions', () => {
    expect(
      filterSlashMenuItems('', { canWrap: true, isWrapped: false }).map(
        (item) => item.name
      )
    ).toEqual(['claim', 'question', 'request', 'evidence', 'protocol'])
    expect(
      filterSlashMenuItems('qu', { canWrap: true, isWrapped: false }).map(
        (item) => item.title
      )
    ).toEqual(['Question'])
  })

  it('only offers unwrap while wrapped', () => {
    expect(
      filterSlashMenuItems('un', { canWrap: true, isWrapped: false })
    ).toEqual([])
    expect(
      filterSlashMenuItems('un', { canWrap: true, isWrapped: true }).map(
        (item) => item.kind
      )
    ).toEqual(['unwrap'])
  })
})
