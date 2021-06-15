import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'

type DocumentEditor = {
  type: 'editor'
} & Document

type DocumentPreview = {
  type: 'preview'
} & Document

export type PaneView = DocumentEditor | DocumentPreview

export type DocumentPane = {
  id: EntityId
  activeView: O.Option<EntityId>
  views: EntityId[]
}

export type NormalizedDocumentPaneStore = {
  entities: {
    panes: Record<EntityId, DocumentPane>
    views: Record<EntityId, PaneView>
  }
  ids: EntityId[]
}
