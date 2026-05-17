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
  .image-container {
    position: relative;
  }

  .image-container img {
    display: block;
    width: var(--image-block-width, auto);
    max-width: var(--image-block-max-width, 100%);
    height: auto;
    margin: 0 auto;
  }

  .content-credentials {
    position: absolute;
    inset-block-start: 0.5rem;
    inset-inline-end: 0.5rem;
    z-index: 2;
    width: 1.75rem;
    height: 1.75rem;
  }

  .content-credentials-pin {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid rgba(17, 24, 39, 0.18);
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.92);
    box-sizing: border-box;
    color: rgb(17, 24, 39);
    box-shadow: none;
    cursor: pointer;
    padding: 0.25rem;
  }

  .content-credentials-pin svg {
    display: block;
    width: 100%;
    height: 100%;
  }

  .content-credentials-pin:hover {
    background: white;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.28);
  }

  .content-credentials-pin:focus-visible {
    background: white;
    outline: 2px solid rgba(14, 165, 233, 0.55);
    outline-offset: 2px;
  }

  .content-credentials-card {
    position: absolute;
    inset-block-start: 0;
    inset-inline-end: 100%;
    width: min(22rem, calc(100vw - 4rem));
    border: 1px solid rgba(17, 24, 39, 0.14);
    border-radius: 8px;
    background: white;
    color: rgb(17, 24, 39);
    box-shadow: 0 12px 32px rgba(17, 24, 39, 0.2);
    box-sizing: border-box;
    font-family: Inter, system-ui, sans-serif;
    font-size: 0.8125rem;
    line-height: 1.35;
    padding: 0.75rem;
    text-align: left;
  }

  .content-credentials-title {
    font-weight: 650;
    margin-block-end: 0.35rem;
  }

  .content-credentials-note {
    color: rgb(75, 85, 99);
    font-size: 0.75rem;
    margin-block-end: 0.6rem;
  }

  .content-credentials-details {
    display: grid;
    gap: 0.2rem;
    margin: 0 0 0.6rem;
  }

  .content-credentials-details div {
    display: grid;
    grid-template-columns: 4.5rem minmax(0, 1fr);
    gap: 0.5rem;
  }

  .content-credentials-details dt {
    color: rgb(107, 114, 128);
    font-weight: 500;
  }

  .content-credentials-details dd {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .content-credentials-section {
    border-block-start: 1px solid rgba(17, 24, 39, 0.1);
    margin-block-start: 0.6rem;
    padding-block-start: 0.6rem;
  }

  .content-credentials-section-title {
    color: rgb(75, 85, 99);
    font-size: 0.72rem;
    font-weight: 650;
    letter-spacing: 0;
    margin-block-end: 0.35rem;
    text-transform: uppercase;
  }

  .content-credentials-list {
    display: grid;
    gap: 0.35rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .content-credentials-list li {
    display: grid;
    align-items: start;
    grid-template-columns: 1rem minmax(0, 1fr);
    gap: 0.45rem;
  }

  .content-credentials-list-icon {
    color: rgb(107, 114, 128);
    display: inline-flex;
    width: 1rem;
    height: 1rem;
    margin-block-start: 0.05rem;
  }

  .content-credentials-list-icon svg {
    display: block;
    width: 1rem;
    height: 1rem;
  }

  .content-credentials-list-content {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 0.25rem 0.45rem;
    min-width: 0;
  }

  .content-credentials-list-main {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .content-credentials-list-meta {
    color: rgb(107, 114, 128);
    font-size: 0.75rem;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .content-credentials-badge {
    border: 1px solid rgba(17, 24, 39, 0.14);
    border-radius: 4px;
    color: rgb(75, 85, 99);
    flex: 0 0 auto;
    font-size: 0.6875rem;
    font-weight: 650;
    line-height: 1.35;
    padding: 0 0.25rem;
  }

  .content-credentials-card a {
    color: rgb(3, 105, 161);
    font-weight: 600;
    text-decoration: none;
  }

  .content-credentials-card a:hover {
    text-decoration: underline;
  }

  .content-credentials-verify-row {
    border-block-start: 1px solid rgba(17, 24, 39, 0.1);
    margin-block-start: 0.7rem;
    padding-block-start: 0.7rem;
  }

  .content-credentials-verify {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    min-height: 1.8rem;
    border: 1px solid rgba(3, 105, 161, 0.28);
    border-radius: 6px;
    background: rgb(240, 249, 255);
    box-sizing: border-box;
    color: rgb(3, 105, 161);
    font-size: 0.75rem;
    font-weight: 650;
    padding: 0.3rem 0.55rem;
    text-decoration: none;
  }

  .content-credentials-verify:hover {
    background: rgb(224, 242, 254);
    text-decoration: none;
  }

  .content-credentials-verify-icon {
    display: inline-flex;
    width: 0.875rem;
    height: 0.875rem;
  }

  .content-credentials-verify-icon svg {
    display: block;
    width: 0.875rem;
    height: 0.875rem;
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
