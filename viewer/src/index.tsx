import 'solid-js'
import { render, renderToString } from 'solid-js/web'
import { DocumentProvider, DocumentSubscriber } from './Document'
import './index.css'

export function Viewer() {
  return (
    <div data-itemscope="root">
      <DocumentSubscriber></DocumentSubscriber>
    </div>
  )
}

render(Viewer, document.getElementById('root') as Node)
