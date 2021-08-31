import { EntityId } from '@reduxjs/toolkit'
import { client } from '../client'
import { state } from '../store'
import {
  patchDocument,
  updateDocument,
} from '../store/documentPane/documentPaneActions'
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

export const saveDocument = async (
  docId: EntityId,
  content: string,
  format?: string
) => {
  if (isTemporaryDocument(state)(docId)) {
    const { value: file } = await client.documents.createFilePath()

    if (file.canceled) return

    const doc = await alterDocument(docId, file.filePath)

    client.documents
      .write({
        documentId: docId,
        content,
        format,
      })
      .then(() => updateDocument({ ...doc, status: 'synced' }))
  } else {
    client.documents
      .write({
        documentId: docId,
        content,
        format,
      })
      .then(() => patchDocument({ id: docId, status: 'synced' }))
  }
}
