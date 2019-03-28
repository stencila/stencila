import * as React from 'react'
import { renderToString } from 'react-dom/server'

exports.replaceRenderer = ({ bodyComponent, replaceBodyHTMLString }) => {
  const ConnectedBody = () => ({ bodyComponent })
  replaceBodyHTMLString(renderToString(<ConnectedBody />))
}
