import { Configurator } from 'substance'
import { SheetPackage, SheetPage, SheetSchema } from 'stencila'
import generateSampleSheet from './generateSampleSheet'

window.addEventListener('load', () => {
  let configurator = new Configurator()
  configurator.import(SheetPackage)
  let xml = generateSampleSheet(100, 20)
  const importer = configurator.createImporter(SheetSchema.getName())
  const sheet = importer.importDocument(xml)
  SheetPage.mount({ sheet }, window.document.body)
})