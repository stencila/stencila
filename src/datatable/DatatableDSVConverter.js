import Converter from '../shared/Converter'

/**
 * Converter to import/export a Datatable from/to a delimiter separated values (DSV) file
 *
 * There are several dialects of [DSV](https://en.wikipedia.org/wiki/Delimiter-separated_values)
 * the best known of which is CSV (comma separated values).
 *
 * Converts to/from Stencila's internal XML buffer format for Datatables
 */
export default class DatatableDSVConverter extends Converter {

  /**
   * @override
   */
  match (path, store) {
    let ext = path.fileName.slice(-4)
    return ['.csv', '.tsv', '.psv'].indexOf(ext) >= 0
  }

  /**
   * @override
   */
  import (buffer, path, storer) {
    
  }

  /**
   * @override
   */
  export (buffer, path, storer) {
    buffer.write()
  }

}
