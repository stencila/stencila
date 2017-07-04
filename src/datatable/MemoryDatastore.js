import Datastore from './Datastore'

/**
 * A `Datastore` implemented in memory
 */
export default class MemoryDatastore extends Datastore {
  
  constructor() {
    super()
    this._data = null
  }

}
