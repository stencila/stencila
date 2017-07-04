import { Document, uuid } from 'substance'
import MemoryDatastore from './MemoryDatastore'

/*
  This model is used to represent a transformation process.
  It applies transforms to the original source.

  Note: this model is a bit different to other models we have
  done with Substance, as it is basically maintaining the source of
  a program instead of changing the data in first place.
  The data is rather kept up2date whenever the transformations have
  changed. Thus, it is similar to Stencila Documents where the cells
  are here the transformations.
*/
export default class DatatableDocument extends Document {

  constructor(...args) {
    super(...args)

    this.store = new MemoryDatastore()
  }

  getStore() {
    return this.store
  }

}