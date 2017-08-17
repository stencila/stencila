import { registerSchema, BasePackage } from 'substance'
import SpreadsheetDocument from './SpreadsheetDocument'
import SpreadsheetSchema from './SpreadsheetSchema'

export default {
  name: 'Spreadsheet',

  configure(config) {
    // registers model nodes and a converter
    registerSchema(config, SpreadsheetSchema, SpreadsheetDocument)

    config.import(BasePackage)

    config.addToolPanel('toolbar', [
      {
        name: 'undo-redo',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['undo-redo']
      },
      {
        name: 'annotations',
        type: 'tool-group',
        showDisabled: true,
        style: 'minimal',
        commandGroups: ['annotations']
      }
    ])

  }
}
