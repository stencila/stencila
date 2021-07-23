import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'

type PaneModule = 'editor' | 'preview'

export type PaneLayout = {
  modules: PaneModule[]
  orientation: 'horizontal' | 'vertical'
}

export type DocumentPane = {
  id: EntityId
  activeView: O.Option<EntityId>
  views: EntityId[]
}

export type NormalizedDocumentPaneStore = {
  activePane: O.Option<EntityId>
  entities: {
    panes: Record<EntityId, DocumentPane>
    layouts: Record<EntityId, PaneLayout>
    documents: Record<EntityId, Document>
  }
  ids: EntityId[]
}
