import Converter from '../shared/Converter'

/**
 * Converter to import/export a Datatable from/to Tabular Data Package (TDP)
 *
 * The [TDP specification](https://specs.frictionlessdata.io/tabular-data-package/)
 * is a [Data Package](https://specs.frictionlessdata.io/data-package/) (represented by a 
 * `datapackage.json` file) that has:
 *
 *  - at least one resource in the resources array
 *  - each resource must be a (Tabular Data Resource)[https://specs.frictionlessdata.io/tabular-data-resource/] (TDR)
 *
 * This converter converts a *single* TDR from a TDP's `datapackage.json`. The TDR can be either:
 *
 * - inline "JSON tabular data" that is array of data rows where each row is an array or object"
 * - a CSV file
 */
export default class DatatableTDPConverter extends Converter {

  /**
   * @override
   */
  match (path, storer) {
    let {dir, file, ext} = Converter.parsePath(path)

    // Is this a `datapackage.json`?
    if (file === 'datapackage.json') return Promise.resolve(true)
    
    // Is this a CSV file with a sibling `datapackage.json`?
    if (ext === 'csv') {
      return storer.readDir(dir).then(files => {
        for (let file of files) {
          if (file === 'datapackage.json') return true
        }
        return false
      })
    }

    // No match
    return Promise.resolve(false)
  }

  /**
   * @override
   */
  import (path, storer, buffer) {
    let {dir, file, ext} = Converter.parsePath(path)

    // Read the `datapackage.json`
    if (file === 'datapackage.json') return Promise.resolve(true)    
  }

  /**
   * @override
   */
  export (path, storer, buffer) {
  }

}
