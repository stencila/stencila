import * as address from './src/address'
import * as value from './src/value'
import * as documentConversion from './src/document/documentConversion'

// ui components
export { default as Dashboard } from './src/dashboard/Dashboard'
export { default as uuid } from './src/util/uuid'

// stubs (needed only by the examples)
export { default as MemoryBackend } from './src/backend/MemoryBackend'
export { default as MemoryBuffer } from './src/backend/MemoryBuffer'

export { default as DocumentEditor } from './src/document/DocumentEditor'
export { default as DocumentPage } from './src/document/DocumentPage'
export { default as DocumentConfigurator } from './src/document/DocumentConfigurator'
export { documentConversion }

export { default as JsContext } from './src/js-context/JsContext'
export { default as functions } from './src/js-context/functions'

export { address, value }
export { default as getQueryStringParam } from './src/util/getQueryStringParam'
