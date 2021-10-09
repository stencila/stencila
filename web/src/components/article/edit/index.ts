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
import { stencilaElement, StencilaElement } from '../../base'
import { articleInputRules } from './inputRules'
import { articleKeymap } from './keymap'
import { articleSchema } from './schema'

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
   * Initialize the custom element by initializing the editor with
   * the content of the `<slot>`, and rendering the editor as a child of
   * this element.
   */
  initialize() {
    if (this.initialized) return
    this.initialized = true

    const sourceElem = this.getSlot(0)

    const editorElem = document.createElement('article')
    editorElem.setAttribute('data-itemscope', 'root')
    editorElem.setAttribute('itemtype', 'http://schema.org/Article')
    editorElem.setAttribute('itemscope', '')
    this.appendChild(editorElem)

    const parser = DOMParser.fromSchema(articleSchema)

    this.doc = parser.parse(sourceElem)

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

    // Tell the store what to call `dispatch` on
    this.view = view

    // Hide the source element
    sourceElem.style.display = 'none'
  }

  /**
   * Receive a new ProseMirror `EditorState` and send a `Patch` to the server.
   *
   * Gets any new document `Step`s, applies them to `this.doc`, transforms each into one or
   * more `Operation`s, and sends them as a `Patch` to the server.
   */
  receiveState(newState: EditorState) {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    // Get any new steps
    const sendable = sendableSteps(newState)
    if (!sendable) return

    const { version, steps, clientID } = sendable

    if (version !== this.version) return

    const ops = []
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

    this.sendPatch({ ops })

    this.version = this.version + steps.length

    // Notify the editor of the new steps and the client ids that
    // are associated with them. Even though the steps came from the same
    // client, the editor, this is necessary to "flush" the sendable steps and increment
    // the version in the editor.
    if (!this.view) throw new Error('Store `view` was not initialized')
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
    // TODO: Currently just using default implem which logs
    super.receiveOperation(op)
    // Pretends to handle the operation, so that some other handler
    // does not modify the ProseMirror managed DOM
    return true
  }

  /**
   * Convert a ProseMirror transaction step to a Stencila document operation
   */
  stepToOperation(step: Step): Operation {
    if (step.constructor === ReplaceStep) {
      const { from, to, slice } = step as ReplaceStepInterface
      if (slice.size !== 0) {
        const value = this.sliceToValue(slice)
        const length = value.length
        if (from === to) {
          return {
            type: 'Add',
            address: this.offsetToAddress(from),
            value,
            length,
          }
        } else {
          return {
            type: 'Replace',
            address: this.offsetToAddress(from),
            items: to - from,
            value,
            length,
          }
        }
      } else {
        return {
          type: 'Remove',
          address: this.offsetToAddress(from),
          items: to - from,
        }
      }
    } else if (step.constructor === ReplaceAroundStep) {
      const { from, to } = step as ReplaceAroundStepInterface
      console.error(`TODO: ReplaceAroundStep ${step}`)
    } else if (step.constructor === AddMarkStep) {
      const { from, to, mark } = step as AddMarkStepInterface
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

    // Get the ProseMirror `ResolvedPos` from the index
    const position = this.doc.resolve(offset)
    let textOffset = position.textOffset

    // For each depth
    const address = []
    for (let depth = 1; depth <= position.depth; depth++) {
      const ancestor = position.node(depth)
      let index = position.index(depth)
      if (
        ancestor.content.childCount > 0 &&
        index >= ancestor.content.childCount
      ) {
        index = ancestor.content.childCount - 1
        textOffset = ancestor.child(index).nodeSize
      }
      if (depth === 1) {
        // The ancestor should be one of the article attributes e.g. `content`, `title`, `abstract`
        address.push(ancestor.type.name)
        address.push(index)
      } else {
        // This ancestor should be a member of the `InlineContent` or `BlockContent` groups, so
        // add the index of the ancestor
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

  /**
   * Convert a ProseMirror step slice to a Stencila operation value
   */
  sliceToValue(slice: Slice): any {
    const { content, openStart, openEnd } = slice

    // A text slice e.g. the value of a keypress
    if (content.childCount === 1 && content.child(0).isText) {
      return content.child(0).textContent
    }

    // A slice resulting from a `splitBlock` operation where `openStart` and `openEnd`
    // indicate the "open depth" at each end of the slice
    if (openStart === openEnd) {
      return '<split>'
    }

    throw new Error(`Unhandled slice type: ${JSON.stringify(slice)}`)
  }
}

stencilaElement('stencila-article')(Article)
