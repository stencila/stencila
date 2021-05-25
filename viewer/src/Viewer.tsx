import { DocumentFetcher, DocumentSubscriber } from './Document'
import { ThemeLinker } from './Theme'

export function Viewer(mode: 'fetch' | 'subscribe' = 'fetch', url?: string) {
  return (
    <ThemeLinker>
      {mode == 'fetch' ? (
        <DocumentFetcher url={url}></DocumentFetcher>
      ) : (
        <DocumentSubscriber url={url}></DocumentSubscriber>
      )}
    </ThemeLinker>
  )
}
