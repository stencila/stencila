import { StateEffect, StateField } from '@codemirror/state'

import { ObjectClient } from '../../../clients/object'

const setObjectClient = StateEffect.define<ObjectClient>()

/**
 * codemirror `StateField` to contain the instance of the
 * 'ObjectClient', allowing the use of its methods within
 * transactions, plugins and updates
 */
const objectClientState = StateField.define<ObjectClient | null>({
  create: () => null,
  update: (objectClient, tr) => {
    for (const e of tr.effects) {
      if (e.is(setObjectClient)) {
        objectClient = e.value
      }
    }
    return objectClient
  },
})

export { objectClientState, setObjectClient }
