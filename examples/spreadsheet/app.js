import { Configurator } from 'substance'
import { SpreadsheetPackage, SpreadsheetPage, SpreadsheetSchema } from 'stencila'
import generateSampleSpreadsheet from './generateSampleSpreadsheet'

window.addEventListener('load', () => {
  let configurator = new Configurator()
  configurator.import(SpreadsheetPackage)
  let xml = generateSampleSpreadsheet(1000, 40)
  const importer = configurator.createImporter(SpreadsheetSchema.getName())
  const sheet = importer.importDocument(xml)
  SpreadsheetPage.mount({
    sheet,
    width: 1400,
    height: 1000
  }, window.document.body)
})
