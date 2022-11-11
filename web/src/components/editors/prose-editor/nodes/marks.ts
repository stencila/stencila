import { MarkSpec } from 'prosemirror-model'

export function emphasis(): MarkSpec {
  return {
    parseDOM: [{ tag: 'em' }, { tag: 'i' }, { style: 'font-style=italic' }],
    toDOM: () => ['em', 0],
  }
}

export function quote(): MarkSpec {
  return {
    parseDOM: [{ tag: 'q' }],
    toDOM: () => ['q', 0],
  }
}

export function strikeout(): MarkSpec {
  return {
    parseDOM: [{ tag: 'del' }],
    toDOM: () => ['del', 0],
  }
}

export function strong(): MarkSpec {
  return {
    parseDOM: [
      { tag: 'strong' },
      { tag: 'b' },
      {
        style: 'font-weight',
        getAttrs: (value) =>
          /^(bold(er)?|[5-9]\d{2,})$/.test(value as string) && null,
      },
    ],
    toDOM: () => ['strong', 0],
  }
}

export function subscript(): MarkSpec {
  return {
    parseDOM: [{ tag: 'sub' }],
    toDOM: () => ['sub', 0],
  }
}

export function superscript(): MarkSpec {
  return {
    parseDOM: [{ tag: 'sup' }],
    toDOM: () => ['sup', 0],
  }
}

export function underline(): MarkSpec {
  return {
    parseDOM: [{ tag: 'u' }],
    toDOM: () => ['u', 0],
  }
}
