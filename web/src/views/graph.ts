import cytoscape, { type Core } from 'cytoscape'
import { LitElement, type PropertyValues, css, html } from 'lit'
import { customElement, property, query, state } from 'lit/decorators'

import { toCytoscapeOptions } from '../graphs/cytoscape'
import { defaultProjectionOptions, projectGraph } from '../graphs/project'
import type {
  Graph,
  GraphLayout,
  GraphProjectionDetail,
  GraphViewPreset,
  ResolvedGraphViewPreset,
} from '../graphs/types'
import { initUno } from '../unocss'
import { buildCytoscapeTheme } from '../utilities/cytoscapeTheme'

import '../site/components/color-mode'

import './graph.css'

initUno()

@customElement('stencila-graph-view')
export class GraphViewElement extends LitElement {
  @property({ attribute: false })
  graph?: Graph

  @state()
  private error?: string

  @state()
  private projection: GraphViewPreset = 'auto'

  @state()
  private layout: GraphLayout = 'breadthfirst'

  @state()
  private detail: GraphProjectionDetail = 'medium'

  @state()
  private includeStructureEdges?: boolean

  @state()
  private includeLowConfidenceEdges = true

  @state()
  private collapseCitationNodes = true

  @state()
  private nodeCount = 0

  @state()
  private edgeCount = 0

  @state()
  private resolvedProjection: ResolvedGraphViewPreset = 'full'

  @state()
  private settingsOpen = false

  @query('.canvas')
  private canvas?: HTMLDivElement

  private cy?: Core

  static override styles = css`
    :host {
      display: block;
      position: relative;
      width: 100vw;
      height: 100vh;
      min-height: 100dvh;
      overflow: hidden;
      background: var(--surface-front, #fff);
      color: var(--text-color, #1f2933);
      font-family: var(--body-font-family, Inter, system-ui, sans-serif);
    }

    .canvas {
      width: 100%;
      height: 100%;
      background: var(--diagram-background, #fff);
    }

    .controls {
      position: absolute;
      top: 0.75rem;
      right: 0.75rem;
      z-index: 1;
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      gap: 0.5rem;
    }

    .control-buttons {
      display: flex;
      gap: 0.5rem;
    }

    .settings-card {
      display: grid;
      gap: 0.8rem;
      width: min(20rem, calc(100vw - 1.5rem));
      padding: 0.85rem;
      border: 1px solid var(--border-color, #d8dee4);
      border-radius: 8px;
      background: var(--surface-front, #fff);
      box-shadow:
        0 16px 32px rgb(15 23 42 / 14%),
        0 2px 8px rgb(15 23 42 / 10%);
    }

    .setting {
      display: grid;
      gap: 0.35rem;
    }

    .setting-label {
      color: var(--text-color-muted, #5d6978);
      font-size: 0.75rem;
      font-weight: 600;
    }

    .checkbox-setting {
      justify-content: start;
    }

    label {
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      font-size: 0.8125rem;
      white-space: nowrap;
    }

    select {
      min-width: 10rem;
      border: 1px solid var(--border-color, #c8d1dc);
      border-radius: 6px;
      padding: 0.35rem 1.75rem 0.35rem 0.5rem;
      background: var(--surface-front, #fff);
      color: inherit;
      font: inherit;
    }

    input[type='checkbox'] {
      width: 1rem;
      height: 1rem;
      margin: 0;
    }

    button {
      display: inline-grid;
      place-items: center;
      width: 2rem;
      height: 2rem;
      border: 1px solid var(--border-color, #c8d1dc);
      border-radius: 6px;
      background: var(--surface-front, #fff);
      color: inherit;
      cursor: pointer;
    }

    .settings-toggle[aria-expanded='true'] {
      background: var(--surface-highlight, #edf2f7);
    }

    button:hover {
      background: var(--surface-highlight, #edf2f7);
    }

    stencila-color-mode .toggle {
      display: inline-grid;
      place-items: center;
      width: 2rem;
      height: 2rem;
      border: 1px solid var(--border-color, #c8d1dc);
      border-radius: 6px;
      background: var(--surface-front, #fff);
      color: inherit;
      cursor: pointer;
      opacity: 1;
    }

    stencila-color-mode .toggle:hover {
      background: var(--surface-highlight, #edf2f7);
    }

    stencila-color-mode .icon {
      display: inline-block;
      width: 1rem;
      height: 1rem;
      background-color: currentColor;
    }

    .actions {
      display: flex;
      gap: 0.5rem;
    }

    .stats {
      color: var(--text-color-muted, #5d6978);
      font-size: 0.8125rem;
      line-height: 1.4;
    }

    .message {
      display: grid;
      place-items: center;
      width: 100%;
      height: 100%;
      padding: 2rem;
      color: var(--text-color-muted, #5d6978);
      text-align: center;
    }

    @media (max-width: 760px) {
      .controls {
        top: 0.5rem;
        right: 0.5rem;
      }

      .settings-card {
        width: calc(100vw - 1rem);
      }

      select {
        width: 100%;
      }
    }
  `

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener(
      'stencila-color-scheme-changed',
      this.onColorSchemeChange
    )
  }

  override firstUpdated() {
    if (!this.graph) {
      this.loadGraph()
    }
  }

  protected override updated(changed: PropertyValues) {
    if (
      changed.has('graph') ||
      changed.has('projection') ||
      changed.has('layout') ||
      changed.has('detail') ||
      changed.has('includeStructureEdges') ||
      changed.has('includeLowConfidenceEdges') ||
      changed.has('collapseCitationNodes')
    ) {
      this.renderGraph()
    }
  }

  override disconnectedCallback() {
    window.removeEventListener(
      'stencila-color-scheme-changed',
      this.onColorSchemeChange
    )
    this.cy?.destroy()
    this.cy = undefined
    super.disconnectedCallback()
  }

  override render() {
    if (this.error) {
      return html`<div class="message">${this.error}</div>`
    }

    return html`
      <div class="canvas" @click=${this.closeSettings}></div>
      <div class="controls" @click=${this.stopClickPropagation}>
        <div class="control-buttons">
          <stencila-color-mode style="icon"></stencila-color-mode>
          <button
            class="settings-toggle"
            title="Graph settings"
            aria-label="Graph settings"
            aria-expanded=${this.settingsOpen}
            @click=${this.toggleSettings}
          >
            <span class="i-lucide:settings"></span>
          </button>
        </div>
        ${this.settingsOpen
          ? html`
              <div class="settings-card">
                <label class="setting">
                  <span class="setting-label">Projection</span>
                  <select
                    .value=${this.projection}
                    @change=${this.onProjectionChange}
                  >
                    <option value="auto">Auto</option>
                    <option value="full">Full</option>
                    <option value="data-flow">Data flow</option>
                    <option value="software-dependencies">
                      Software dependencies
                    </option>
                    <option value="citations">Citations</option>
                    <option value="reactivity">Reactivity</option>
                  </select>
                </label>
                <label class="setting">
                  <span class="setting-label">Layout</span>
                  <select .value=${this.layout} @change=${this.onLayoutChange}>
                    <option value="breadthfirst">Breadthfirst</option>
                    <option value="cose">Force</option>
                    <option value="grid">Grid</option>
                    <option value="circle">Circle</option>
                  </select>
                </label>
                <label class="setting">
                  <span class="setting-label">Detail</span>
                  <select .value=${this.detail} @change=${this.onDetailChange}>
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                  </select>
                </label>
                <label class="checkbox-setting">
                  <input
                    type="checkbox"
                    .checked=${this.effectiveIncludeStructureEdges()}
                    @change=${this.onStructureChange}
                  />
                  Structure
                </label>
                <label class="checkbox-setting">
                  <input
                    type="checkbox"
                    .checked=${this.includeLowConfidenceEdges}
                    @change=${this.onConfidenceChange}
                  />
                  Low confidence
                </label>
                <label class="checkbox-setting">
                  <input
                    type="checkbox"
                    .checked=${this.collapseCitationNodes}
                    @change=${this.onCitationCollapseChange}
                  />
                  Collapse citations
                </label>
                <div class="actions">
                  <button title="Fit" aria-label="Fit" @click=${this.fit}>
                    <span class="i-lucide:maximize"></span>
                  </button>
                  <button
                    title="Reset layout"
                    aria-label="Reset layout"
                    @click=${this.renderGraph}
                  >
                    <span class="i-lucide:refresh-cw"></span>
                  </button>
                </div>
                <span class="stats">
                  ${this.resolvedProjection} - ${this.nodeCount} nodes -
                  ${this.edgeCount} edges
                </span>
              </div>
            `
          : null}
      </div>
    `
  }

  private loadGraph() {
    const data = document.getElementById('stencila-graph-data')?.textContent

    if (!data) {
      this.error = 'No graph data found for this page.'
      return
    }

    try {
      this.graph = JSON.parse(data) as Graph
    } catch (error) {
      this.error = error instanceof Error ? error.message : 'Unable to parse graph data.'
    }
  }

  private renderGraph = () => {
    if (!this.graph || !this.canvas) {
      return
    }

    this.cy?.destroy()

    const view = projectGraph(this.graph, {
      ...defaultProjectionOptions(this.projection),
      detail: this.detail,
      includeStructureEdges: this.includeStructureEdges,
      includeLowConfidenceEdges: this.includeLowConfidenceEdges,
      collapseCitationNodes: this.collapseCitationNodes,
    })

    this.nodeCount = view.nodes.length
    this.edgeCount = view.edges.length
    this.resolvedProjection = view.preset

    const theme = buildCytoscapeTheme(this)
    this.cy = cytoscape(toCytoscapeOptions(view, this.canvas, this.layout, theme))
  }

  private fit = () => {
    this.cy?.fit(undefined, 40)
  }

  private onColorSchemeChange = () => {
    this.renderGraph()
  }

  private toggleSettings = () => {
    this.settingsOpen = !this.settingsOpen
  }

  private closeSettings = () => {
    this.settingsOpen = false
  }

  private stopClickPropagation = (event: Event) => {
    event.stopPropagation()
  }

  private onProjectionChange = (event: Event) => {
    const projection = (event.currentTarget as HTMLSelectElement)
      .value as GraphViewPreset

    this.projection = projection
    this.includeStructureEdges = undefined
  }

  private onLayoutChange = (event: Event) => {
    this.layout = (event.currentTarget as HTMLSelectElement).value as GraphLayout
  }

  private onDetailChange = (event: Event) => {
    this.detail = (event.currentTarget as HTMLSelectElement)
      .value as GraphProjectionDetail
  }

  private onStructureChange = (event: Event) => {
    this.includeStructureEdges = (event.currentTarget as HTMLInputElement).checked
  }

  private onConfidenceChange = (event: Event) => {
    this.includeLowConfidenceEdges = (event.currentTarget as HTMLInputElement).checked
  }

  private onCitationCollapseChange = (event: Event) => {
    this.collapseCitationNodes = (event.currentTarget as HTMLInputElement).checked
  }

  private effectiveIncludeStructureEdges(): boolean {
    if (this.includeStructureEdges !== undefined) {
      return this.includeStructureEdges
    }

    const preset =
      this.projection === 'auto' ? this.resolvedProjection : this.projection
    return defaultProjectionOptions(preset).includeStructureEdges ?? false
  }
}
