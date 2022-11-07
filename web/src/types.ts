import { ViewUpdate } from '@codemirror/view'
import { ValidatorTypes } from '@stencila/schema'
import { DocumentClient } from './clients/document-client'

declare global {
  interface Window {
    stencilaConfig: DocumentConfig
    stencilaClient: DocumentClient
    stencilaShellTerminal: (elementId: string, dir: string) => void
  }
}

export interface DocumentConfig {
  origin?: string
  path?: string
  token?: string
  documentId?: string
  mode?: string
  executableLanguages?: string[]
}

export type ElementId = string

/**
 * The id of a node within a document
 */
export type NodeId = string

/**
 * Possible document subscription topics
 */
export type DocumentTopic = 'patched' | 'kernel-monitoring'

/**
 * The browser event emitted when a document is patched (e.g. by the
 * WYSIWYG article editor)
 */
export interface DocumentPatchEvent extends CustomEvent {
  detail: {
    patch: Patch
    then: Then
  }
}

/**
 * The browser event emitted when the language of a text editor
 * is changed
 */
export interface LanguageChangeEvent extends CustomEvent {
  detail: {
    ext: string
    name: string
  }
}

/**
 * The browser event emitted when the content of a text editor
 * is changed
 */
export interface ContentChangeEvent extends CustomEvent {
  detail: ViewUpdate | string
}

/**
 * The browser event emitted when a property of a `CallArgument` changes.
 */
export interface CallArgumentChangeEvent extends CustomEvent {
  detail: {
    index: number
    property: 'symbol' | 'value'
    value: string
  }
}

/**
 * The browser event emitted when a property of a `Call` changes.
 */
export interface CallChangeEvent extends CustomEvent {
  detail: {
    property: 'source' | 'select'
    value: string
  }
}

/**
 * The browser event emitted when either the type or property of a parameter validator changes.
 */
export interface ValidatorChangeEvent extends CustomEvent {
  detail:
    | {
        type: 'property'
        name: string
        value: string
      }
    | {
        type: 'validator'
        value: Exclude<ValidatorTypes['type'], 'Validator'>
      }
}

/**
 * The browser event emitted when either the name of value of the parameter changes.
 */
export interface ParameterChangeEvent extends CustomEvent {
  detail: {
    property: 'name' | 'value'
    value: string
  }
}

export type Then = {
  assemble?: When
  compile?: When
  execute?: When
  write?: When
}

/**
 * When compile, execute and write operations should
 * be done after a patch
 */
export type When = 'Now' | 'Soon' | 'Never'

// The following type definitions were generated from Rust types (via the `schemars` crate)
// This needs updating but was moved here from the `../../node` module to avoid a dependency
// on that. It is likely that a new `../../typescript` module be created specifically for these
// type definitions and those generated from `../../schema`.

/**
 * Used to determine various application behaviors e.g. not reading binary formats into memory unnecessarily
 */
export interface Format {
  /**
   * Whether or not this is a known format (ie.e. not automatically created)
   */
  known: boolean
  /**
   * The lowercase name of the format e.g. `md`, `docx`, `dockerfile`
   */
  name: string
  /**
   * Whether or not the format should be considered binary e.g. not to be displayed in a text / code editor
   */
  binary: boolean
  /**
   * Whether HTML previews are normally supported for documents of this format. See also `Document.previewable` which indicates whether a HTML preview is supported for a particular document.
   */
  preview: boolean
  /**
   * Any additional extensions (other than it's name) that this format should match against.
   */
  extensions: string[]
}

export type Kernel =
  | {
      type: 'Default'
    }
  | {
      type: 'Calc'
    }

/**
 * An in-memory representation of a document
 */
export interface Document {
  /**
   * The document identifier
   */
  id: string
  /**
   * The absolute path of the document's file.
   */
  path: string
  /**
   * The project directory for this document.
   *
   * Used to restrict file links (e.g. image paths) to within the project for both security and reproducibility reasons. For documents opened from within a project, this will be project directory. For "orphan" documents (opened by themselves) this will be the parent directory of the document. When the document is compiled, an error will be returned if a file link is outside of the root.
   */
  project: string
  /**
   * Whether or not the document's file is in the temporary directory.
   */
  temporary: boolean
  /**
   * The synchronization status of the document. This is orthogonal to `temporary` because a document's `content` can be synced or un-synced with the file system regardless of whether or not its `path` is temporary..
   */
  status: 'synced' | 'unwritten' | 'unread' | 'deleted'
  /**
   * The name of the document
   *
   * Usually the filename from the `path` but "Untitled" for temporary documents.
   */
  name: string
  /**
   * The format of the document.
   *
   * On initialization, this is inferred, if possible, from the file name extension of the document's `path`. However, it may change whilst the document is open in memory (e.g. if the `load` function sets a different format).
   */
  format: Format
  /**
   * Whether a HTML preview of the document is supported
   *
   * This is determined by the type of the `root` node of the document. Will be `true` if the `root` is a type for which HTML previews are implemented e.g. `Article`, `ImageObject` and `false` if the `root` is `None`, or of some other type e.g. `Entity`.
   *
   * This flag is intended for dynamically determining whether to open a preview panel for a document by default. Regardless of its value, a user should be able to open a preview panel, in HTML or some other format, for any document.
   */
  previewable: boolean
  /**
   * Addresses of nodes in `root` that have an `id`
   *
   * Used to fetch a particular node (and do something with it like `patch` or `execute` it) rather than walking the node tree looking for it. It is necessary to use [`Address`] here (rather than say raw pointers) because pointers or references will change as the document is patched. These addresses are shifted when the document is patched to account for this.
   */
  addresses: Record<string, Address>
  /**
   * The set of relations between this document, or nodes in this document, and other resources.
   *
   * Relations may be external (e.g. this document links to another file) or internal (e.g. the second code chunk uses a variable defined in the first code chunk).
   */
  relations?: Record<string, [Relation, Resource]>
  /**
   * Keeping track of client ids per topics allows for a some optimizations. For example, events will only be published on topics that have at least one subscriber.
   *
   * Valid subscription topics are the names of the `DocumentEvent` types:
   *
   * - `removed`: published when document file is deleted - `renamed`: published when document file is renamed - `modified`: published when document file is modified - `encoded:<format>` published when a document's content is changed internally or externally and  conversions have been completed e.g. `encoded:html`
   */
  subscriptions: {
    [k: string]: string[]
  }
  /**
   * The kernels in the document kernel space
   */
  kernels: {
    [k: string]: Kernel
  }
  /**
   * The symbols in the document kernel space
   */
  symbols: {
    [k: string]: SymbolInfo
  }
}
export interface SymbolInfo {
  /**
   * The type of the object that the symbol refers to (e.g `Number`, `Function`)
   *
   * Should be used as a hint only, to the underlying, native type of the symbol.
   */
  kind: string
  /**
   * The home kernel of the symbol
   *
   * The home kernel of a symbol is the kernel that it was last assigned in. As such, a symbol's home kernel can change, although this is discouraged.
   */
  home: string
  /**
   * The time that the symbol was last assigned in the home kernel
   *
   * A symbol is considered assigned when  a `CodeChunk` with an `Assign` relation to the symbol is executed or the `kernel.set` method is called.
   */
  assigned: string
  /**
   * A timestamp is recorded for each time that a symbol is mirrored to another kernel. This allows unnecessary mirroring to be avoided if the symbol has not been assigned since it was last mirrored to that kernel.
   */
  mirrored: {
    [k: string]: string
  }
}

export interface DocumentEvent {
  /**
   * The type of event
   */
  type: 'deleted' | 'renamed' | 'modified' | 'patched' | 'encoded'
  /**
   * The document associated with the event
   */
  document: Document
  /**
   * The content associated with the event, only provided for, `modified` and `encoded` events.
   */
  content?: string
  /**
   * The format of the document, only provided for `modified` (the format of the document) and `encoded` events (the format of the encoding).
   */
  format?: Format
  /**
   * The `Patch` associated with a `Patched` event
   */
  patch?: Patch
}

export type PatchesSchema =
  | {
      Slot: Slot
    }
  | {
      Address: Address
    }
  | {
      Patch: Patch
    }
  | {
      Operation: Operation
    }
/**
 * A slot, used as part of an [`Address`], to locate a value within a `Node` tree.
 *
 * Slots can be used to identify a part of a larger object.
 *
 * The `Name` variant can be used to identify:
 *
 * - the property name of a `struct` - the key of a `HashMap<String, ...>`
 *
 * The `Integer` variant can be used to identify:
 *
 * - the index of a `Vec` - the index of a Unicode character in a `String`
 *
 * The `None` variant is used in places where a `Slot` is required but none applies to the particular type or use case.
 *
 * In contrast to JSON Patch, which uses a [JSON Pointer](http://tools.ietf.org/html/rfc6901) to describe the location of additions and removals, slots offer improved performance and type safety.
 */
export type Slot = number | string
/**
 * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
 *
 * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
 *
 * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
 */
export type Address = Slot[]
/**
 * The operations that can be used in a patch to mutate one node into another.
 *
 * These are the same operations as described in [JSON Patch](http://jsonpatch.com/) (with the exception of `copy` and `test`). Note that `Replace` and `Move` could be represented by combinations of `Remove` and `Add`. They are included as a means of providing more semantically meaningful patches, and more space efficient serializations (e.g. it is not necessary to represent the value being moved or copied).
 *
 * In addition, there is a `Transform` operation which can be used describe the transformation of a node to another type, having a similar structure. Examples includes:
 *
 * - a `String` to an `Emphasis` - a `Paragraph` to a `QuoteBlock` - a `CodeChunk` to a `CodeBlock`
 *
 * The `length` field on `Add` and `Replace` is not necessary for applying operations, but is useful for generating them and for determining if there are conflicts between two patches without having to downcast the `value`.
 *
 * Note that for `String`s the integers in `address`, `items` and `length` all refer to Unicode characters not bytes.
 */
export type Operation =
  | OperationAdd
  | OperationRemove
  | OperationReplace
  | OperationMove
  | OperationTransform

/**
 * A set of [`Operation`]s
 */
export interface Patch {
  /**
   * The [`Operation`]s to apply
   */
  ops: Operation[]
  /**
   * The id of the node to which to apply this patch
   */
  target?: string
  /**
   * The id of the actor that generated this patch e.g. a web browser client, or file watcher
   */
  actor?: string
  address?: Slot[]
  version?: number
}
/**
 * Add a value
 */
export interface OperationAdd {
  type: 'Add'
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  address: Slot[]
  /**
   * The value to add
   */
  value: any
  /**
   * The number of items added
   */
  length: number
  /**
   * The HTML encoding of `value`
   */
  html?: string
}
/**
 * Remove one or more values
 */
export interface OperationRemove {
  type: 'Remove'
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  address: Slot[]
  /**
   * The number of items to remove
   */
  items: number
}
/**
 * Replace one or more values
 */
export interface OperationReplace {
  type: 'Replace'
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  address: Slot[]
  /**
   * The number of items to replace
   */
  items: number
  /**
   * The replacement value
   */
  value: any
  /**
   * The number of items added
   */
  length: number
  /**
   * The HTML encoding of `value`
   */
  html?: string
}
/**
 * Move a value from one address to another
 */
export interface OperationMove {
  type: 'Move'
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  from: Slot[]
  /**
   * The number of items to move
   */
  items: number
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  to: Slot[]
}
/**
 * Transform a value from one type to another
 */
export interface OperationTransform {
  type: 'Transform'
  /**
   * The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
   *
   * Implemented as a double-ended queue. Given that addresses usually have less than six slots it may be more performant to use a stack allocated `tinyvec` here instead.
   *
   * Note: This could instead have be called a "Path", but that name was avoided because of potential confusion with file system paths.
   */
  address: Slot[]
  /**
   * The type of `Node` to transform from
   */
  from: string
  /**
   * The type of `Node` to transform to
   */
  to: string
}

/**
 * An implementation, and extension, of schema.org [`Project`](https://schema.org/Project). Uses schema.org properties where possible but adds extension properties where needed (e.g. `theme`).
 */
export interface Project {
  /**
   * The name of the project
   */
  name?: string
  /**
   * A description of the project
   */
  description?: string
  /**
   * The path (within the project) of the project's image
   *
   * If not specified, will default to the most recently modified image in the project (if any).
   */
  image?: string
  /**
   * The path (within the project) of the project's main file
   *
   * If not specified, will default to the first file matching the the regular expression in the configuration settings.
   */
  main?: string
  /**
   * The default theme to use when viewing documents in this project
   *
   * If not specified, will default to the default theme in the configuration settings.
   */
  theme?: string
  /**
   * A list of project sources and their destination within the project
   */
  sources?: SourceDestination[]
  /**
   * A list of file conversions
   */
  conversions?: Conversion[]
  /**
   * Glob patterns for paths to be excluded from file watching
   *
   * As a performance optimization, paths that match these patterns are excluded from file watching updates. If not specified, will default to the patterns in the configuration settings.
   */
  watchExcludePatterns?: string[]
  /**
   * The filesystem path of the project folder
   */
  path: string
  /**
   * The resolved path of the project's image file
   */
  imagePath?: string
  /**
   * The resolved path of the project's main file
   */
  mainPath?: string
  /**
   * The files in the project folder
   */
  files: Record<string, File>
  /**
   * The project's dependency graph
   */
  graph: Graph
}
/**
 * The definition of a conversion between files within a project
 */
export interface Conversion {
  /**
   * The path of the input document
   */
  input?: string
  /**
   * The path of the output document
   */
  output?: string
  /**
   * The format of the input (defaults to being inferred from the file extension of the input)
   */
  from?: string
  /**
   * The format of the output (defaults to being inferred from the file extension of the output)
   */
  to?: string
  /**
   * Whether or not the conversion is active
   */
  active?: boolean
}

export interface ProjectEvent {
  /**
   * The project associated with the event
   */
  project: Project
  /**
   * The type of event
   */
  type: 'updated'
}

/**
 * A file or directory within a `Project`
 */
export interface File {
  /**
   * The absolute path of the file or directory
   */
  path: string
  /**
   * The name of the file or directory
   */
  name: string
  /**
   * Time that the file was last modified (Unix Epoch timestamp)
   */
  modified?: number
  /**
   * Size of the file in bytes
   */
  size?: number
  /**
   * Format of the file
   *
   * Usually this is the lower cased filename extension (if any) but may also be normalized. May be more convenient, and usually more available, than the `media_type` property.
   */
  format: string
  /**
   * The parent `File`, if any
   */
  parent?: string
  /**
   * If a directory, a list of the canonical paths of the files within it. Otherwise, `None`.
   *
   * A `BTreeSet` rather than a `Vec` so that paths are ordered without having to be resorted after insertions. Another option is `BinaryHeap` but `BinaryHeap::retain` is  only on nightly and so is awkward to use.
   */
  children?: string[]
}

/**
 * These events published under the `projects:<project-path>:files` topic.
 */
export interface FileEvent {
  /**
   * The path of the project (absolute)
   */
  project: string
  /**
   * The path of the file (absolute)
   *
   * For `renamed` events this is the _old_ path.
   */
  path: string
  /**
   * The type of event e.g. `Refreshed`, `Modified`, `Created`
   *
   * A `refreshed` event is emitted when the entire set of files is updated.
   */
  type: 'refreshed' | 'created' | 'removed' | 'renamed' | 'modified'
  /**
   * The updated file
   *
   * Will be `None` for for `refreshed` and `removed` events, or if for some reason it was not possible to fetch metadata about the file.
   */
  file?: File
  /**
   * The updated set of files in the project
   *
   * Represents the new state of the file tree after the event including updated `parent` and `children` properties of files affects by the event.
   */
  files: Record<string, File>
}

/**
 * A resource in a dependency graph (the nodes of the graph)
 */
export type Resource =
  | {
      type: 'Symbol'
      /**
       * The path of the file that the symbol is defined in
       */
      path: string
      /**
       * The name/identifier of the symbol
       */
      name: string
      /**
       * The type of the object that the symbol refers to (e.g `Number`, `Function`)
       *
       * Should be used as a hint only, and as such is excluded from equality and hash functions.
       */
      kind: string
    }
  | {
      type: 'Node'
      /**
       * The path of the file that the node is defined in
       */
      path: string
      /**
       * The id of the node with the document
       */
      id: string
      /**
       * The type of node e.g. `Parameter`, `CodeChunk`
       */
      kind: string
    }
  | {
      type: 'File'
      /**
       * The path of the file
       */
      path: string
    }
  | {
      type: 'Source'
      /**
       * The name of the project source
       */
      name: string
    }
  | {
      type: 'Module'
      /**
       * The programming language of the module
       */
      language: string
      /**
       * The name of the module
       */
      name: string
    }
  | {
      type: 'Url'
      /**
       * The URL of the external resource
       */
      url: string
    }

/**
 * The relation between two resources in a dependency graph (the edges of the graph)
 *
 * Some relations carry additional information such whether the relation is active (`Import` and `Convert`) or the range that they occur in code (`Assign`, `Use`, `Read`) etc
 */
export type Relation =
  | {
      type: 'Assign'
      /**
       * The range within code that the assignment is done
       */
      range: [number, number, number, number]
    }
  | {
      type: 'Convert'
      /**
       * Whether or not the conversion is automatically updated
       */
      auto: boolean
    }
  | {
      type: 'Embed'
      [k: string]: unknown
    }
  | {
      type: 'Import'
      /**
       * Whether or not the import is automatically updated
       */
      auto: boolean
    }
  | {
      type: 'Include'
      [k: string]: unknown
    }
  | {
      type: 'Link'
      [k: string]: unknown
    }
  | {
      type: 'Read'
      /**
       * The range within code that the read is declared
       */
      range: [number, number, number, number]
    }
  | {
      type: 'Use'
      /**
       * The range within code that the use is declared
       */
      range: [number, number, number, number]
    }
  | {
      type: 'Write'
      /**
       * The range within code that the write is declared
       */
      range: [number, number, number, number]
    }

/**
 * A subject-relation-object triple
 */
export type Triple = [Resource, Relation, Resource]

/**
 * A project dependency graph
 */
export interface Graph {
  /**
   * The resources in the graph
   */
  nodes: Resource[]
  /**
   * The relations between resources in the graph
   */
  edges: {
    from: 'integer'
    to: 'integer'
    relation: Resource
  }[]
}

export interface GraphEvent {
  /**
   * The path of the project (absolute)
   */
  project: string
  /**
   * The type of event
   */
  type: 'updated'
  /**
   * The graph at the time of the event
   */
  graph: Graph
}

/**
 * A session
 */
export interface Session {
  /**
   * The id of the session
   */
  id: string
  /**
   * The id of the project that this session is for
   */
  project: string
  /**
   * The id of the snapshot that this session is for
   */
  snapshot: string
  /**
   * This is an optimization to avoid collecting session metrics and / or publishing events if there are no clients subscribed.
   */
  subscriptions: {
    [k: string]: string[]
  }
  /**
   * The status of the session
   */
  status: 'Pending' | 'Starting' | 'Started' | 'Stopping' | 'Stopped'
}

/**
 * A session event
 */
export type SessionEvent =
  | {
      type: 'Updated'
      session: Session
    }
  | {
      type: 'Heartbeat'
      session: Session
    }

/**
 * Each source by destination combination should be unique to a project. It is possible to have the same source being imported to multiple destinations within a project and for multiple sources to used the same destination (e.g. the root directory of the project).
 */
export interface SourceDestination {
  /**
   * The source from which files will be imported
   */
  source?:
    | {
        type: 'Null'
      }
    | {
        type: 'Elife'
        /**
         * Number of the article
         */
        article: number
      }
    | {
        type: 'GitHub'
        /**
         * Owner of the repository
         */
        owner: string
        /**
         * Name of the repository
         */
        name: string
        /**
         * Path within the repository
         */
        path?: string
      }
  /**
   * The destination path within the project
   */
  destination?: string
  /**
   * Whether or not the source is active
   *
   * If the source is active an import job will be created for it each time the project is updated.
   */
  active?: boolean
  /**
   * A list of file paths currently associated with the source, relative to the project root
   */
  files?: string[]
}

/**
 * As far as possible using existing properties defined in schema.org [`SoftwareApplication`](https://schema.org/SoftwareApplication) but extensions added where necessary.
 */
export interface Plugin {
  /**
   * The name of the plugin
   */
  name: string
  /**
   * The version of the plugin
   */
  softwareVersion: string
  /**
   * A description of the plugin
   */
  description: string
  /**
   * URL of the image to be used when displaying the plugin
   */
  image?: string
  /**
   * A list of URLS that the plugin can be installed from
   */
  installUrl: string[]
  /**
   * A list of plugin "features" Each feature is a `JSONSchema` object describing a method (including its parameters).
   */
  featureList: true[]
  /**
   * If the plugin is installed, the installation type
   */
  installation?: 'docker' | 'binary' | 'javascript' | 'python' | 'r' | 'link'
  /**
   * The last time that the plugin manifest was updated. Used to determine if a refresh is necessary.
   */
  refreshed?: string
  /**
   * The next version of the plugin, if any.
   *
   * If the plugin is installed and there is a newer version of the plugin then this property should be set at the time of refresh.
   */
  next?: Plugin
  /**
   * The current alias for this plugin, if any
   */
  alias?: string
}

/**
 * Plugin installation method
 *
 * Which method to use to install a plugin.
 */
export type PluginInstallation =
  | 'docker'
  | 'binary'
  | 'javascript'
  | 'python'
  | 'r'
  | 'link'

/**
 * An enumeration of custom errors returned by this library
 *
 * Where possible functions should return one of these errors to provide greater context to the user, in particular regarding actions that can be taken to resolve the error.
 */
export type Error =
  | {
      type: 'InvalidUUID'
      family: string
      id: string
      message: string
    }
  | {
      type: 'NotSame'
      message: string
      [k: string]: unknown
    }
  | {
      type: 'NotEqual'
      message: string
      [k: string]: unknown
    }
  | {
      type: 'UnpointableType'
      address: Address
      type_name: string
      message: string
    }
  | {
      type: 'InvalidAddress'
      type_name: string
      details: string
      message: string
    }
  | {
      type: 'InvalidPatchOperation'
      op: string
      type_name: string
      message: string
    }
  | {
      type: 'InvalidPatchValue'
      type_name: string
      message: string
    }
  | {
      type: 'InvalidSlotVariant'
      variant: string
      type_name: string
      message: string
    }
  | {
      type: 'InvalidSlotName'
      name: string
      type_name: string
      message: string
    }
  | {
      type: 'InvalidSlotIndex'
      index: number
      type_name: string
      message: string
    }
  | {
      type: 'UnknownFormat'
      format: string
      message: string
    }
  | {
      type: 'IncompatibleLanguage'
      language: string
      kernel_type: string
      message: string
    }
  | {
      type: 'UndelegatableMethod'
      method: Method
      message: string
    }
  | {
      type: 'UndelegatableCall'
      method: Method
      params: {
        [k: string]: unknown
      }
      message: string
    }
  | {
      type: 'PluginNotInstalled'
      plugin: string
      message: string
    }
  | {
      type: 'Unspecified'
      message: string
    }
/**
 * An enumeration of all methods
 */
export type Method =
  | 'import'
  | 'export'
  | 'decode'
  | 'encode'
  | 'coerce'
  | 'reshape'
  | 'compile'
  | 'build'
  | 'execute'

export const FORMATS: Record<string, Format> = {
  '3gp': {
    known: true,
    name: '3gp',
    binary: true,
    preview: true,
    extensions: [],
  },
  dir: {
    known: true,
    name: 'dir',
    binary: true,
    preview: false,
    extensions: [],
  },
  dockerfile: {
    known: true,
    name: 'dockerfile',
    binary: false,
    preview: false,
    extensions: [],
  },
  docx: {
    known: true,
    name: 'docx',
    binary: true,
    preview: true,
    extensions: [],
  },
  flac: {
    known: true,
    name: 'flac',
    binary: true,
    preview: true,
    extensions: [],
  },
  gif: {
    known: true,
    name: 'gif',
    binary: true,
    preview: true,
    extensions: [],
  },
  html: {
    known: true,
    name: 'html',
    binary: false,
    preview: true,
    extensions: [],
  },
  ipynb: {
    known: true,
    name: 'ipynb',
    binary: false,
    preview: true,
    extensions: [],
  },
  jpg: {
    known: true,
    name: 'jpg',
    binary: true,
    preview: true,
    extensions: ['jpeg'],
  },
  js: {
    known: true,
    name: 'js',
    binary: false,
    preview: false,
    extensions: [],
  },
  json: {
    known: true,
    name: 'json',
    binary: false,
    preview: true,
    extensions: [],
  },
  json5: {
    known: true,
    name: 'json5',
    binary: false,
    preview: true,
    extensions: [],
  },
  latex: {
    known: true,
    name: 'latex',
    binary: false,
    preview: true,
    extensions: ['tex'],
  },
  makefile: {
    known: true,
    name: 'makefile',
    binary: false,
    preview: false,
    extensions: [],
  },
  md: {
    known: true,
    name: 'md',
    binary: false,
    preview: true,
    extensions: [],
  },
  mp3: {
    known: true,
    name: 'mp3',
    binary: true,
    preview: true,
    extensions: [],
  },
  mp4: {
    known: true,
    name: 'mp4',
    binary: true,
    preview: true,
    extensions: [],
  },
  odt: {
    known: true,
    name: 'odt',
    binary: true,
    preview: true,
    extensions: [],
  },
  ogg: {
    known: true,
    name: 'ogg',
    binary: true,
    preview: true,
    extensions: [],
  },
  ogv: {
    known: true,
    name: 'ogv',
    binary: true,
    preview: true,
    extensions: [],
  },
  png: {
    known: true,
    name: 'png',
    binary: true,
    preview: true,
    extensions: [],
  },
  py: {
    known: true,
    name: 'py',
    binary: false,
    preview: false,
    extensions: [],
  },
  r: {
    known: true,
    name: 'r',
    binary: false,
    preview: false,
    extensions: [],
  },
  rmd: {
    known: true,
    name: 'rmd',
    binary: false,
    preview: true,
    extensions: [],
  },
  rpng: {
    known: true,
    name: 'rpng',
    binary: true,
    preview: true,
    extensions: [],
  },
  sh: {
    known: true,
    name: 'sh',
    binary: false,
    preview: false,
    extensions: [],
  },
  toml: {
    known: true,
    name: 'toml',
    binary: false,
    preview: true,
    extensions: [],
  },
  ts: {
    known: true,
    name: 'ts',
    binary: false,
    preview: false,
    extensions: [],
  },
  txt: {
    known: true,
    name: 'txt',
    binary: false,
    preview: false,
    extensions: [],
  },
  webm: {
    known: true,
    name: 'webm',
    binary: true,
    preview: true,
    extensions: [],
  },
  xml: {
    known: true,
    name: 'xml',
    binary: false,
    preview: true,
    extensions: [],
  },
  yaml: {
    known: true,
    name: 'yaml',
    binary: false,
    preview: true,
    extensions: [],
  },
}
