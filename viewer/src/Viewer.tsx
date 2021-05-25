import { DocumentFetcher, DocumentSubscriber } from './Document'
import { ThemeSwitcher } from './Theme'

export function Viewer(mode: 'fetch' | 'subscribe' = 'fetch', url?: string) {
  return (
    <ThemeSwitcher>
      {mode == 'fetch' ? (
        <DocumentFetcher url={url}></DocumentFetcher>
      ) : (
        <DocumentSubscriber url={url}></DocumentSubscriber>
      )}
    </ThemeSwitcher>
  )
}
