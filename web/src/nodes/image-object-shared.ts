import { css } from 'lit'

/**
 * Media types for JS-based visualizations
 *
 * Single source of truth used by both `image-object.ts` and `image-object-static.ts`.
 */
export const MEDIA_TYPES = {
  cytoscape: 'application/vnd.cytoscape.v3+json',
  echarts: 'application/vnd.apache.echarts+json',
  leaflet: 'text/html',
  mermaid: 'text/vnd.mermaid',
  plotly: 'application/vnd.plotly.v1+json',
  vegaLite: 'application/vnd.vegalite.v5+json',
} as const

/**
 * Shared styles for ImageObject components
 *
 * Used by both `image-object.ts` (full version) and `image-object-static.ts`
 * to avoid style duplication and drift.
 */
export const imageObjectStyles = css`
  :host {
    display: block;
  }

  /* Container for all chart visualizations (ECharts, Plotly, Vega-Lite, Cytoscape) */
  .viz-container {
    width: 100%;
    aspect-ratio: var(--plot-aspect-ratio);
    min-height: var(--plot-height-min);
    max-height: var(--plot-height-max);
  }

  /* Plotly needs a padder wrapper for consistent padding */
  .plotly-padder {
    padding: var(--plot-padding-top) var(--plot-padding-right)
      var(--plot-padding-bottom) var(--plot-padding-left);
    background-color: var(--plot-background);
    box-sizing: border-box;
  }

  /* Mermaid diagram container */
  .mermaid-container {
    display: flex;
    justify-content: center;
  }

  /* Reset SVG styles for Mermaid diagrams */
  .mermaid-container svg {
    line-height: 1;
    font-size: inherit;
    font-family: inherit;
    text-align: initial;
    letter-spacing: normal;
    word-spacing: normal;
    margin: 0;
    padding: 0;
    border: none;
    display: block;
    max-width: 100%;
    height: auto;
  }

  /* Fix Mermaid edge label backgrounds
   * Based on solution from: https://stephenkernan.com/blog/how-to-style-mermaid-edge-labels */
  .mermaid-container svg foreignObject:has(.edgeLabel) {
    background-color: transparent;
  }
  .mermaid-container svg foreignObject:has(.edgeLabel) .edgeLabel,
  .mermaid-container svg foreignObject:has(.edgeLabel) .labelBkg {
    background-color: transparent !important;
    font-size: 97.5%;
  }

  /* Cytoscape graph container (square aspect ratio) */
  .cytoscape-container {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
  }

  /* Leaflet map iframe */
  .leaflet-iframe {
    width: 100%;
    aspect-ratio: var(--plot-aspect-ratio, 16/9);
    min-height: var(--plot-height-min, 300px);
    max-height: var(--plot-height-max, 600px);
    border: none;
  }

  /* Block-level image styling.
   * Vertical spacing is owned by the host element in the light DOM (images.css).
   * The shadow DOM only handles rendering-level properties. */
  .image-container img {
    display: block;
    width: var(--image-block-width, auto);
    max-width: var(--image-block-max-width, 100%);
    height: auto;
    margin: 0 auto;
  }

  /* Error display */
  .error {
    color: red;
    padding: 1rem;
    border: 1px solid red;
    border-radius: 4px;
    background: #fee;
  }
`
