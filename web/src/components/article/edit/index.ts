import { Address, DomOperation, Operation, Patch } from '@stencila/stencila'
import { collab, receiveTransaction, sendableSteps } from 'prosemirror-collab'
import { baseKeymap } from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { gapCursor } from 'prosemirror-gapcursor'
import { history } from 'prosemirror-history'
import { keymap } from 'prosemirror-keymap'
import { DOMParser, Mark, Node, Slice } from 'prosemirror-model'
import { EditorState, Transaction } from 'prosemirror-state'
import {
  AddMarkStep,
  RemoveMarkStep,
  ReplaceAroundStep,
  ReplaceStep,
  Step,
} from 'prosemirror-transform'
import { EditorView } from 'prosemirror-view'
import {
  isArray,
  isObject,
  JsonValue,
} from '../../../patches/checks'
import { diff } from '../../../patches/json'
import { stencilaElement, StencilaElement } from '../../base'
import { articleInputRules } from './inputRules'
import { articleKeymap } from './keymap'
import { articleMarks, articleSchema } from './schema'

// The following interfaces were necessary because the way they are defined
// in @types/prosemirror-transform (as classes with only constructors) does
// not seem to permit typed access to properties

interface ReplaceStepInterface extends Step {
  from: number
  to: number
  slice: Slice
  structure?: boolean
}

interface ReplaceAroundStepInterface extends Step {
  from: number
  to: number
  gapFrom: number
  gapTo: number
  slice: Slice
  insert: number
  structure?: boolean
}

interface AddMarkStepInterface extends Step {
  from: number
  to: number
  mark: Mark
}

interface RemoveMarkStepInterface extends Step {
  from: number
  to: number
  mark: Mark
}

export class Article extends StencilaElement {
  initialized = false

  version = 0

  doc?: Node

  view?: EditorView

  static hydrate() {
    StencilaElement.hydrate(this, 'http://schema.org/Article')
  }

  /**
   * Initialize the custom element by parsing the `<article>` in the `<slot>`,
   * rendering the editor as a child of this element, and hiding (or removing)
   * the original `<article>` element.
   */
  initialize() {
    // Avoid recursion triggered by slotchange event
    if (this.initialized) return
    this.initialized = true

    // Get the source <article> element and hide it
    const sourceElem = this.getSlot(0)
    sourceElem.style.display = 'none'

    // Parse the source into a document and hold onto it
    // so that we can use it to map reconcile and map operations
    const parser = DOMParser.fromSchema(articleSchema)
    this.doc = parser.parse(sourceElem)

    // DEBUG: Get an initial
    nodeToJSON(this.doc)

    // Create the editor state
    const state = EditorState.create({
      schema: articleSchema,
      doc: this.doc,
      plugins: [
        // Locally defined input rules and keymap
        articleInputRules,
        keymap(articleKeymap),
        // Plugin that enables undo and redo
        history(),
        // Plugin for collaboration that talks to the store
        collab({ version: this.version }),
        // The `baseKeymap` contains "bindings not specific to any schema" e.g.
        // `Enter` (to split paragraphs, add a newline to a code blocks),
        // `Mod-Enter` (to exit code block), `Backspace`, `Delete` etc etc.
        // These can be overridden above.
        keymap(baseKeymap),
        // Plugin that "causes a decoration to show up at the drop position when something is dragged over the editor"
        dropCursor({ class: 'drop-cursor' }),
        // Plugin that provides "a block-level cursor that can be used to focus places that don't allow regular selection"
        gapCursor(),
      ],
    })

    // Create the editor <article> element
    const editorElem = document.createElement('article')
    editorElem.setAttribute('data-itemscope', 'root')
    editorElem.setAttribute('itemtype', 'http://schema.org/Article')
    editorElem.setAttribute('itemscope', '')
    this.appendChild(editorElem)

    // Render the editor view
    const me = this
    const view = new EditorView(editorElem, {
      state,
      dispatchTransaction(transaction) {
        // Send any new state to the view and the store
        const newState = view.state.apply(transaction)
        view.updateState(newState)
        me.receiveState(newState)
      },
    })

    // Hold on to the view so that transactions can be dispatched to it
    this.view = view
  }

  /**
   * Receive a new ProseMirror `EditorState` and send a `Patch` to the server.
   *
   * Obtains any new document `Step`s associated with the new state,
   * applies them to `this.doc`, transforms each into one or
   * more `Operation`s, and sends them as a `Patch` to the server.
   */
  receiveState(newState: EditorState) {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    // Get any new steps
    const sendable = sendableSteps(newState)
    if (!sendable) return

    const { version, steps, clientID } = sendable

    // TODO: instead of ignoring this, is some sort of reset required
    if (version !== this.version) return

    const pre = nodeToJSON(this.doc)

    let ops = []
    for (const step of steps) {
      try {
        const op = this.stepToOperation(step)
        ops.push(op)
      } catch (error) {
        console.log(error)
      }

      const { failed, doc } = step.apply(this.doc)
      if (typeof failed === 'string') {
        console.error(failed)
      } else if (doc) {
        this.doc = doc
      }
    }

    const post = nodeToJSON(this.doc)
    const patch = diff(pre, post)
    this.sendPatch(patch)

    this.version = this.version + steps.length

    // Notify the editor of the new steps and the client ids that
    // are associated with them. Even though the steps came from the same
    // client, the editor, this is necessary to "flush" the sendable steps and increment
    // the version in the editor.
    if (!this.view) throw new Error('Article `view` was not initialized')
    this.view.dispatch(
      receiveTransaction(
        this.view.state,
        steps,
        steps.map(() => clientID)
      )
    )
  }

  /**
   * Receive a Stencila `Operation` from the server.
   *
   * Transforms each `Operation` into a ProseMirror `Step` and sends them to the editor.
   */
  receiveOperation(op: DomOperation) {
    // Pretends to handle the operation, so that some other handler
    // does not modify the ProseMirror managed DOM
    console.warn('TODO: Incoming patch operations are not currently handled')
    return true
  }

  /**
   * Convert a ProseMirror transaction `Step` to a Stencila document `Operation`.
   */
  stepToOperation(step: Step): Operation {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    if (step.constructor === ReplaceStep) {
      const { from, to, slice } = step as ReplaceStepInterface
      let address = this.offsetToAddress(from)
      if (slice.size !== 0) {
        // Slice has content, so adding or replacing
        const { content, openStart, openEnd } = slice
        console.log(from, to, content, openStart, openEnd)

        let value: any = ''
        if (openStart === 0 && openStart === 0) {
          // Adding a character e.g. a keypress
          if (content.childCount === 1 && content.child(0).isText) {
            value = content.child(0).textContent
          } else {
            throw new Error('Unexpected content')
          }
        } else if (content.size > 1 && openStart > 0 && openEnd == openStart) {
          // Splitting e.g. pressing Enter within or at end of a paragraph
          // Need to replace the existing node with two new ones containing the
          // child nodes before and after the split
          // So go up to the blocks and add one
          address = address.slice(0, -3)
          const last = (address[address.length - 1] as number) + 1
          address = [...address.slice(0, -1), last]
          // TODO split content
          value = [{ type: 'Paragraph', content: [''] }]
        }

        const length = value.length
        if (from === to) {
          return {
            type: 'Add',
            address,
            value,
            length,
          }
        } else {
          return {
            type: 'Replace',
            address,
            items: to - from,
            value,
            length,
          }
        }
      } else {
        // Slice is empty, so removing
        return {
          type: 'Remove',
          address,
          items: to - from,
        }
      }
    } else if (step.constructor === ReplaceAroundStep) {
      const { from, to } = step as ReplaceAroundStepInterface
      console.error(`TODO: ReplaceAroundStep ${step}`)
    } else if (step.constructor === AddMarkStep) {
      const { from, to, mark } = step as AddMarkStepInterface
      console.log('from', from, this.offsetToAddress(from))
      console.log('to', to, this.offsetToAddress(to))
      console.log('mark', mark.type.name)
      // Replace the surrounding node with up to three nodes:
      // 1. the content before (if from address is greater than 0)
      // 2. the new mark
      // 3. the content after
      const fromAddress = this.offsetToAddress(from)

      const node = this.doc.nodeAt(from)
      if (!node) throw new Error('Unexpected nullish node')
      const text = node.text
      if (!text) throw new Error('Unexpected nullish text')

      const position = this.doc.resolve(from)
      const parent = position.parent
      console.log(parent.content)

      const before = text.slice(0, 2)
      const marked = text.slice(2, 3)
      const after = text.slice(3)
      return {
        type: 'Replace',
        address: fromAddress.slice(0, -1),
        items: 1,
        value: [before, { type: mark.type.name, content: [marked] }, after],
        length: 3,
      }
      return {
        type: 'Transform',
        address: this.offsetToAddress(from),
        // @ts-expect-error because not yet a property
        items: to - from,
        from: 'String',
        to: mark.type.name,
      }
    } else if (step.constructor === RemoveMarkStep) {
      const { from, to, mark } = step as RemoveMarkStepInterface
      return {
        type: 'Transform',
        address: this.offsetToAddress(from),
        // @ts-expect-error because not yet a property
        items: to - from,
        from: mark.type.name,
        to: 'String',
      }
    }

    // Should be unreachable as the above handle all step types
    throw new Error(`Unhandled step type ${JSON.stringify(step)}`)
  }

  /**
   * Convert a Stencila document operation to a ProseMirror document transaction
   */
  operationToStep(op: DomOperation): Transaction {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    const transaction = new Transaction(this.doc)
    switch (op.type) {
      case 'Add': {
        const { address, html } = op
        return transaction.insert(this.addressToOffset(address), [])
      }
      default: {
        // TODO: All the other operation types
        return transaction
      }
    }
  }

  /**
   * Convert a ProseMirror document offset into a Stencila document address.
   *
   * For relevant ProseMirror documentation see:
   *   - https://prosemirror.net/docs/guide/#doc.indexing
   *   - https://prosemirror.net/docs/ref/#model.Resolved_Positions
   */
  offsetToAddress(offset: number): Address {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    // Get the ProseMirror `ResolvedPos` from the offset
    const position = this.doc.resolve(offset)
    let textOffset = position.textOffset

    // For each depth level that the position is in the node tree
    // calculate the Stencila slot(s) o add to the address
    const address = []
    for (let depth = 1; depth <= position.depth; depth++) {
      const ancestor = position.node(depth)
      let index = position.index(depth)

      // This seems to be necessary for when characters are being added to the
      // end of a paragraph
      if (
        ancestor.content.childCount > 0 &&
        index >= ancestor.content.childCount
      ) {
        index = ancestor.content.childCount - 1
        textOffset = ancestor.child(index).nodeSize
      }

      if (depth === 1) {
        // At this depth, the ancestor should be one of the article attributes
        // e.g. `content`, `title`, `abstract`
        address.push(ancestor.type.name)
        address.push(index)
      } else {
        // At other depths, the ancestor will have a `contentProp` (defaulting to 'content')
        // that the index points to
        address.push(ancestor.type.spec.contentProp ?? 'content')
        address.push(index)
      }
    }

    // If the position has ProseMirror marks the we need to add additional nesting to
    // the address
    // TODO
    // for (const mark in position.marks()) {
    // address.push('content')
    // address.push(0)
    // }

    address.push(textOffset)
    return address
  }

  /**
   * Convert a Stencila document address to a ProseMirror document offset.
   */
  addressToOffset(address: Address): number {
    // TODO calculate offsets
    return 0
  }
}

/**
 * Convert the ProseMirror document to Stencila JSON.
 *
 * This is used to generate Stencila patch `Operations` from more complicated
 * ProseMirror transactions which are difficult to transform into operations directly.
 */
function nodeToJSON(node: Node) {
  const json = node.toJSON()
  console.log('JSON: ', JSON.stringify(json, null, '  '))
  const transformed = transformJSON(json)
  console.log('Transformed: ', JSON.stringify(transformed, null, '  '))
  return transformed
}

/**
 * Transform a ProseMirror JSON representation of a document node
 * into a Stencila Schema representation.
 *
 * @why To generate a Stencila `Patch` for a ProseMirror transformation we
 * first need to represent the document as a Stencila document.
 *
 * @how Performance is important given that this function is recursively
 * called over potentially large documents. Given that, it favours mutation
 * and loops over restructuring and mapping etc.
 */
function transformJSON(value: JsonValue): JsonValue {
  if (typeof value === 'string') return value
  if (typeof value === 'number') return value
  if (typeof value === 'boolean') return value
  if (value === null) return value

  if (Array.isArray(value)) {
    // Transform items of array and then merge adjacent inlines that mary have
    // arisen from how ProseMirror marks are handled (see below)
    let index = 0
    let prev: JsonValue | undefined
    while (index < value.length) {
      const curr = transformJSON(value[index] as JsonValue)
      if (
        isObject(prev) &&
        isArray(prev.content) &&
        isObject(curr) &&
        isArray(curr.content) &&
        prev.type == curr.type &&
        articleMarks.includes(curr.type as string)
      ) {
        value.splice(index, 1)
        prev.content.push(...curr.content)
      } else {
        value[index] = curr
        prev = curr
        index++
      }
    }
    return value
  }

  // Transform properties of objects
  for (const key in value) {
    value[key] = transformJSON(value[key] as JsonValue)
  }

  switch (value.type) {
    case 'text': {
      // Transform ProseMirror text nodes into a (possibly nested) set of
      // inline nodes e.g. String, Strong, Emphasis.
      // Note that with this algorithm, the first applied mark will be the outer one.
      // This is related to the above merging of inline nodes.
      const text = value as {
        text: string
        marks?: [{ type: string }]
      }
      let node: string | { type: string; content: [JsonValue] } = text.text
      if (text.marks) {
        for (const mark of text.marks) {
          node = {
            type: mark.type,
            content: [node],
          }
        }
      }
      return node
    }
    case 'Paragraph': {
      // Ensure that the `content` property is defined
      // (wont be for empty paragraphs etc).
      // Important for diffing
      if (value.content === undefined) {
        value.content = []
      }
      return value
    }
    case 'Article':
      // Reshape the top-level article
      return {
        type: 'Article',
        // @ts-ignore
        content: value.content[0].content,
      }
    default:
      return value
  }
}

stencilaElement('stencila-article')(Article)
