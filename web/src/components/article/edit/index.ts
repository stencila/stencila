import { Address, DomOperation, Operation } from '@stencila/stencila'
import { collab, receiveTransaction, sendableSteps } from 'prosemirror-collab'
import { baseKeymap } from 'prosemirror-commands'
import { dropCursor } from 'prosemirror-dropcursor'
import { gapCursor } from 'prosemirror-gapcursor'
import { history } from 'prosemirror-history'
import { keymap } from 'prosemirror-keymap'
import { DOMParser, Mark, Node, ResolvedPos, Slice } from 'prosemirror-model'
import { EditorState, Transaction } from 'prosemirror-state'
import { ReplaceStep, Step } from 'prosemirror-transform'
import { EditorView } from 'prosemirror-view'
import { isNumber, JsonValue } from '../../../patches/checks'
import { applyPatch, diff } from '../../../patches/json'
import { stencilaElement, StencilaElement } from '../../base'
import { prosemirrorToStencila } from './convert'
import { articleInputRules } from './inputRules'
import { articleKeymap } from './keymap'
import { articleSchema } from './schema'

// Import ProseMirror's `EditorView` styles for correct whitespace handling etc
import 'prosemirror-view/style/prosemirror.css'
import { CodeBlockView } from './components/codeBlockView'

// The following interfaces were necessary because the way they are defined
// in @types/prosemirror-transform (as classes with only constructors) does
// not seem to permit typed access to properties

interface ReplaceStepInterface extends Step {
  from: number
  to: number
  slice: Slice
  structure?: boolean
}

interface _ReplaceAroundStepInterface extends Step {
  from: number
  to: number
  gapFrom: number
  gapTo: number
  slice: Slice
  insert: number
  structure?: boolean
}

interface _AddMarkStepInterface extends Step {
  from: number
  to: number
  mark: Mark
}

interface _RemoveMarkStepInterface extends Step {
  from: number
  to: number
  mark: Mark
}

export class Article extends StencilaElement {
  initialized = false

  version = 0

  doc?: Node

  root?: JsonValue

  view?: EditorView

  static hydrate(): void {
    StencilaElement.hydrate(this, 'http://schema.org/Article')
  }

  /**
   * Initialize the custom element by parsing the `<article>` in the `<slot>`,
   * rendering the editor as a child of this element, and hiding (or removing)
   * the original `<article>` element.
   */
  initialize(): void {
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
    this.root = prosemirrorToStencila(this.doc)

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
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const me = this
    const view = new EditorView(editorElem, {
      state,
      dispatchTransaction(transaction) {
        // Send any new state to the view and the store
        const newState = view.state.apply(transaction)
        view.updateState(newState)
        me.receiveState(newState)
      },
      nodeViews: {
        CodeBlock: (node, view, getPos) =>
          new CodeBlockView(node, view, getPos),
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
  receiveState(newState: EditorState): void {
    if (this.doc === undefined)
      throw new Error('The `doc` has not been initialized')
    if (this.root === undefined)
      throw new Error('The `root` has not been initialized')

    // Get any new steps
    const sendable = sendableSteps(newState)
    if (!sendable) return

    const { version, steps, clientID } = sendable

    // TODO: instead of ignoring this, is some sort of reset required
    if (version !== this.version) return

    // Each step is converted to an operation, which is then applied to `this.root`,
    // and the step is applied to the ProseMirror document
    const ops: Operation[] = []
    for (const step of steps) {
      // Attempt to generate an operation from the step
      // TODO: enable this call when stepToOperation is fixed
      const op = undefined // this.stepToOperation(step)

      // Apply the step this `this.doc` so it is up to date and
      // so that positions in the step can be correctly resolved to addresses.
      const { failed, doc } = step.apply(this.doc)
      if (typeof failed === 'string') {
        console.error(failed)
      } else if (doc) {
        this.doc = doc
      }

      // If necessary, generate operations from the change in the `this.root`,
      // otherwise apply the operation so that `this.root` stays up to date
      if (op === undefined) {
        console.debug('⚠️ Generating patch from diff')
        // TODO: do diff on the smallest part of the doc possible e.g a single paragraph
        const newRoot = prosemirrorToStencila(this.doc)
        const patch = diff(this.root, newRoot)
        ops.push(...patch.ops)
        this.root = newRoot
      } else {
        ops.push(op)
        try {
          applyPatch(this.root, {
            ops: ops.map((op): DomOperation => {
              // @ts-expect-error because this is a temporary until we unify Operation and DomOperation
              // eslint-disable-next-line
              return { ...op, json: op.value }
            }),
          })
        } catch (error) {
          // There was an error applying the patch so recover by setting root to
          // the current state of the document
          console.error(error)
          this.root = prosemirrorToStencila(this.doc)
        }
      }
    }

    this.sendPatch({ ops })

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
  receiveOperation(_op: DomOperation): boolean {
    // Pretends to handle the operation, so that some other handler
    // does not modify the ProseMirror managed DOM
    console.warn('TODO: Incoming patch operations are not currently handled')
    return true
  }

  /**
   * Convert a ProseMirror transaction `Step` to a Stencila patch `Operation`.
   *
   * Return `undefined` if the conversion has not been implemented yet, in which
   * case diff-based operations will be created.
   */
  stepToOperation(step: Step): Operation | undefined {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    if (step.constructor === ReplaceStep) {
      const { from, to, slice } = step as ReplaceStepInterface
      if (slice.size !== 0) {
        // Slice has content, so adding or replacing
        const { content, openStart, openEnd } = slice
        if (openStart === 0 && openEnd === 0) {
          // Adding text e.g. a keypress or pasting
          if (content.childCount === 1 && content.child(0).isText) {
            const position = this.doc.resolve(from)
            const address = this.offsetToAddress(position)
            if (address === undefined) return

            const value = content.child(0).textContent
            const length = value.length
            if (from === to) {
              if (position.parent.childCount === 0) {
                // The parent does not have any children yet, so add the value as the
                // first child of the parent
                return {
                  type: 'Add',
                  address: address.slice(0, -1),
                  value: [value],
                  length,
                }
              } else {
                // Add the text to the existing text node
                return {
                  type: 'Add',
                  address,
                  value,
                  length,
                }
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
            console.log('Unexpected content')
          }
        }
      } else {
        // Slice is empty, so removing something.
        // If the parent node (e.g. a Paragraph) is the same for the `to` and `from`
        // positions then do the remove with the items calculated form the addresses
        const fromPos = this.doc.resolve(from)
        const toPos = this.doc.resolve(to)
        if (fromPos.parent === toPos.parent) {
          const address = this.offsetToAddress(fromPos)
          if (address === undefined) return

          const toAddress = this.offsetToAddress(toPos)
          if (toAddress === undefined) return

          const lastFrom = address[address.length - 1]
          const lastTo = toAddress[toAddress.length - 1]
          if (!(isNumber(lastFrom) && isNumber(lastTo))) return

          const items = lastTo - lastFrom
          if (items > 0) {
            return {
              type: 'Remove',
              address,
              items,
            }
          }
        }
      }
    }
  }

  /**
   * Convert a Stencila document operation to a ProseMirror document transaction
   */
  operationToStep(op: DomOperation): Transaction {
    if (!this.doc) throw new Error('The `doc` has not been initialized')

    const transaction = new Transaction(this.doc)
    switch (op.type) {
      case 'Add': {
        const { address } = op
        return transaction.insert(this.addressToOffset(address), [])
      }
      default: {
        // TODO: All the other operation types
        return transaction
      }
    }
  }

  /**
   * Convert a ProseMirror `ResolvedPos` into a Stencila document address.
   *
   * For relevant ProseMirror documentation see:
   *   - https://prosemirror.net/docs/guide/#doc.indexing
   *   - https://prosemirror.net/docs/ref/#model.Resolved_Positions
   */
  offsetToAddress(position: ResolvedPos): Address | undefined {
    // Check that there are no ProseMirror nodes before this one in the parent
    // node that have multiple marks as that will invalidate the address
    // This may be able to be dealt with; rather than throwing a wobbly like this
    for (let index = 0; index < position.index(position.depth); index++) {
      if (position.parent.content.child(index).marks.length > 1) {
        return
      }
    }

    let textOffset = position.textOffset

    // For each depth level that the position is in the node tree
    // calculate the Stencila slot(s) to add to the address
    const address: Address = []
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

    // If there are marks on this node then we need to add to the address
    const marks = position.marks()
    if (marks.length === 1) {
      address.push('content')
      address.push(0)
    } else if (marks.length > 1) {
      // Currently unable to determine address if more than one mark
      return
    }

    address.push(textOffset)
    return address
  }

  /**
   * Convert a Stencila document address to a ProseMirror document offset.
   */
  addressToOffset(_address: Address): number {
    // TODO calculate offsets
    return 0
  }
}

stencilaElement('stencila-article')(Article)
