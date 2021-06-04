import { EntityId } from '@reduxjs/toolkit'
import { Document } from 'stencila'
import { option as O } from 'fp-ts'

type DocumentEditor = {
  type: 'editor'
} & Document

type DocumentPreview = {
  type: 'preview'
  format: 'string'
} & Document

export type DocumentView = DocumentEditor | DocumentPreview

export type DocumentPane = {
  id: EntityId
  activeView: O.Option<Document['id']>
  views: Document['id'][]
}

export type NormalizedDocumentPaneStore = {
  entities: {
    panes: Record<EntityId, DocumentPane>
    views: Record<EntityId, DocumentView>
  }
  ids: EntityId[]
}
