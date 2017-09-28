import { Configurator } from 'substance'

import FunctionPackage from './FunctionPackage'
import FunctionSchema from './FunctionSchema'

export default function (main, files) {
  let configurator = new Configurator()
  configurator.import(FunctionPackage)
  const importer = configurator.createImporter(FunctionSchema.getName())
  const xml = importer.compileDocument(main, files)
  const func = importer.importDocument(xml)
  return func
}

