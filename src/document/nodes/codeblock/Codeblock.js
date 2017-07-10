import { BlockNode } from 'substance'

export default class Codeblock extends BlockNode {}

Codeblock.define({
  type: 'codeblock',
  language: { type: 'string', default: 'text' },
  source: { type: 'string', default: '' }
})
