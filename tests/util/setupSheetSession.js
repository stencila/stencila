import createRawArchive from './createRawArchive'
import loadRawArchive from './loadRawArchive'

export default function setupSheetSession(sheetData, engine) {
  let context = { engine }
  let rawArchive = createRawArchive([ sheetData ])
  let archive = loadRawArchive(rawArchive, context)
  let sheetSession = archive.getEditorSession('sheet')
  let sheet = sheetSession.getDocument()
  return { archive, sheetSession, sheet, engine }
}