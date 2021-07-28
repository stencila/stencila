import { EntityId } from '@reduxjs/toolkit'
import { client } from '../client'
import { state } from '../store'
import { updateDocument } from '../store/documentPane/documentPaneActions'
import { isTemporaryDocument } from '../store/documentPane/documentPaneSelectors'

export const alterDocument = async (
  docId: EntityId,
  path?: string,
  format?: string
) => {
  const { value: doc } = await client.documents.alter(docId, path, format)
  updateDocument(doc)
  return doc
}

export const saveDocument = async (docId: EntityId, content: string) => {
  if (isTemporaryDocument(state)(docId)) {
    const { value: filePath } = await client.documents.createFilePath()

    if (!filePath) return

    const doc = await alterDocument(docId, filePath)

    client.documents.write({
      documentId: docId,
      content,
    })

    return updateDocument(doc)
  } else {
    client.documents.write({
      documentId: docId,
      content,
    })
  }
}
