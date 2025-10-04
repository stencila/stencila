import { html } from 'lit'
import { css } from '@twind/core'

/**
 * Compile Leaflet HTML map content
 *
 * Validates that the HTML content contains Leaflet map indicators
 */
export async function compileLeaflet(
  contentUrl: string,
  onSuccess: (htmlContent: string) => void,
  onError: (error: string) => void
): Promise<void> {
  try {
    // Check if the HTML content contains Leaflet indicators
    const htmlContent = contentUrl
    const isLeafletMap =
      htmlContent.includes('leaflet') ||
      htmlContent.includes('L.map') ||
      htmlContent.includes('leaflet.js')

    if (isLeafletMap) {
      onSuccess(htmlContent)
    } else {
      onError('HTML content does not appear to contain a valid map')
    }
  } catch (error) {
    onError(error.message ?? error.toString())
  }
}

/**
 * Render Leaflet HTML map in an iframe
 */
export function renderLeafletIframe(htmlContent: string) {
  const mapStyles = css`
    & iframe {
      width: 100%;
      height: 400px;
      border: none;
    }
  `

  // Create a blob URL for the HTML content to safely render in an iframe
  const blob = new Blob([htmlContent], { type: 'text/html' })
  const blobUrl = URL.createObjectURL(blob)

  return html`
    <div slot="content" class="overflow-x-auto">
      <div class=${mapStyles}>
        <iframe src=${blobUrl} sandbox="allow-scripts allow-same-origin"></iframe>
      </div>
    </div>
  `
}
