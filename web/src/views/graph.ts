import {
  type VirtualElement,
  computePosition,
  flip,
  offset,
  shift,
} from '@floating-ui/dom'
import cytoscape, {
  type Core,
  type EdgeSingular,
  type EventObject,
  type NodeSingular,
} from 'cytoscape'
import { LitElement, type PropertyValues, css, html } from 'lit'
import { customElement, property, query, state } from 'lit/decorators'
import { keyed } from 'lit/directives/keyed.js'

import { toCytoscapeOptions } from '../graphs/cytoscape'
import {
  type BadgeTone,
  type PresentedDetail,
  actionBadge,
  actionLabel,
  confidenceBadgeTone,
  edgeSummary,
  evidenceBadgeTone,
  evidenceLabel,
  formatValue,
  humanizeLabel,
  isLongMetadataValue,
  isRecord,
  metadataIdentity,
  metadataTypeLabel,
  nodeBadgeTone,
  nodeProperties,
  presentActions,
  presentEvidence,
  relationshipBadgeTone,
  sortedDetails,
  webUrlLabel,
} from '../graphs/evidence'
import { dependencyNeighborhood } from '../graphs/lineage'
import { defaultProjectionOptions, projectGraph } from '../graphs/project'
import type {
  Graph,
  GraphLayout,
  GraphLayoutSpacing,
  GraphProjectionDetail,
  GraphView,
  GraphViewEdge,
  GraphViewNode,
  GraphViewPreset,
  ResolvedGraphViewPreset,
} from '../graphs/types'
import { initUno } from '../unocss'
import { buildCytoscapeTheme } from '../utilities/cytoscapeTheme'

import '../site/components/color-mode'

import './graph.css'

initUno()

const layoutSpacings: GraphLayoutSpacing[] = [
  'compact',
  'cozy',
  'balanced',
  'open',
  'spacious',
]

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
  private layoutSpacing: GraphLayoutSpacing = 'cozy'

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

  @state()
  private pinnedEdgeId?: string

  @state()
  private selectedNodeId?: string

  @state()
  private hoveredEdgeId?: string

  @state()
  private tracedNodeId?: string

  @state()
  private legendOpen = window.innerWidth > 760

  @query('.canvas')
  private canvas?: HTMLDivElement

  @query('.edge-preview')
  private edgePreview?: HTMLDivElement

  private cy?: Core
  private view?: GraphView
  private edgesById = new Map<string, GraphViewEdge>()
  private hoverTimer?: number
  private hoverPoint = { x: 0, y: 0 }

  static override styles = css`
    :host {
      display: block;
      position: relative;
      width: 100vw;
      height: 100vh;
      min-height: 100dvh;
      overflow: hidden;
      background: var(--surface-background, #fff);
      color: var(--text-color-primary, #1f2933);
      color-scheme: light;
      font-family: var(--body-font-family, Inter, system-ui, sans-serif);
    }

    :host([data-color-scheme='dark']) {
      color-scheme: dark;
    }

    .canvas {
      width: 100%;
      height: 100%;
      background-color: var(--diagram-background, #fff);
      background-image:
        linear-gradient(
          color-mix(in srgb, var(--text-color-muted, #64748b) 9%, transparent) 1px,
          transparent 1px
        ),
        linear-gradient(
          90deg,
          color-mix(in srgb, var(--text-color-muted, #64748b) 9%, transparent) 1px,
          transparent 1px
        );
      background-size: 24px 24px;
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

    .controls.inspector-visible {
      right: calc(min(27rem, 92vw) + 0.75rem);
    }

    .control-buttons,
    .actions {
      display: flex;
      gap: 0.5rem;
    }

    .settings-card {
      display: grid;
      gap: 0.8rem;
      width: min(20rem, calc(100vw - 1.5rem));
      padding: 0.85rem;
      border: 1px solid var(--border-color-default, #d8dee4);
      border-radius: 9px;
      background: color-mix(
        in srgb,
        var(--surface-elevated, #fff) 94%,
        transparent
      );
      box-shadow:
        0 16px 32px rgb(15 23 42 / 14%),
        0 2px 8px rgb(15 23 42 / 10%);
      backdrop-filter: blur(12px);
    }

    .setting {
      display: grid;
      gap: 0.35rem;
    }

    .setting-label,
    .stats,
    .muted {
      color: var(--text-color-muted, #5d6978);
    }

    .setting-label {
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
      border: 1px solid var(--border-color-default, #c8d1dc);
      border-radius: 6px;
      padding: 0.35rem 1.75rem 0.35rem 0.5rem;
      background: var(--surface-raised, #fff);
      color: inherit;
      font: inherit;
    }

    .range-setting {
      grid-template-columns: 1fr auto;
    }

    .range-setting input {
      grid-column: 1 / -1;
      width: 100%;
      margin: 0;
      accent-color: var(--color-accent, #0f766e);
    }

    .range-setting output {
      color: var(--text-color-primary, #1f2933);
      font-size: 0.75rem;
      text-transform: capitalize;
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
      border: 1px solid var(--border-color-default, #c8d1dc);
      border-radius: 7px;
      background: var(--surface-elevated, #fff);
      color: inherit;
      cursor: pointer;
      transition:
        background 120ms ease,
        transform 120ms ease;
    }

    .settings-toggle[aria-expanded='true'],
    button:hover {
      background: var(--surface-raised, #edf2f7);
    }

    button:hover {
      transform: translateY(-1px);
    }

    stencila-color-mode .toggle {
      display: inline-grid;
      place-items: center;
      width: 2rem;
      height: 2rem;
      border: 1px solid var(--border-color-default, #c8d1dc);
      border-radius: 7px;
      background: var(--surface-elevated, #fff);
      color: inherit;
      cursor: pointer;
      opacity: 1;
    }

    stencila-color-mode .icon {
      display: inline-block;
      width: 1rem;
      height: 1rem;
      background-color: currentColor;
    }

    .stats {
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

    .legend {
      position: absolute;
      top: 0.75rem;
      left: 0.75rem;
      z-index: 1;
      width: fit-content;
      max-width: calc(100vw - 1.5rem);
      border: 1px solid var(--border-color-default, #d8dee4);
      border-radius: 10px;
      background: color-mix(
        in srgb,
        var(--surface-elevated, #fff) 94%,
        transparent
      );
      box-shadow: 0 6px 18px rgb(15 23 42 / 10%);
      color: var(--text-color-secondary, #475569);
      font-size: 0.75rem;
      backdrop-filter: blur(12px);
      overflow: hidden;
    }

    .legend[open] {
      width: min(23rem, calc(100vw - 1.5rem));
    }

    .legend > summary {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: baseline;
      gap: 0.5rem;
      padding: 0.6rem 0.75rem;
      cursor: pointer;
      list-style: none;
    }

    .legend > summary::-webkit-details-marker {
      display: none;
    }

    .legend[open] > summary {
      border-bottom: 1px solid var(--border-color-muted, #e5e9ee);
    }

    .legend-title {
      color: var(--text-color-primary, #1f2933);
      font-weight: 700;
    }

    .legend-chevron {
      display: inline-block;
      width: 0.45rem;
      height: 0.45rem;
      border-right: 1.5px solid currentColor;
      border-bottom: 1.5px solid currentColor;
      transform: translateY(-0.1rem) rotate(45deg);
      transition: transform 120ms ease;
    }

    .legend[open] .legend-chevron {
      transform: translateY(0.1rem) rotate(225deg);
    }

    .legend-groups {
      display: grid;
      grid-template-columns: 1fr;
      gap: 0.75rem;
      max-height: calc(100dvh - 4.1rem);
      padding: 0.75rem;
      overflow: auto;
      overscroll-behavior: contain;
    }

    .legend-group {
      min-width: 0;
      margin: 0;
      border-top: 1px solid var(--border-color-muted, #e5e9ee);
      padding: 0.75rem 0 0;
    }

    .legend-group:first-child {
      border-top: 0;
      padding-top: 0;
    }

    .legend-group-title {
      margin: 0 0 0.55rem;
      color: var(--text-color-primary, #1f2933);
      font-size: 0.68rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.055em;
    }

    .legend-list {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 0.5rem 0.75rem;
      margin: 0;
      padding: 0;
      list-style: none;
    }

    .legend-item {
      display: grid;
      grid-template-columns: 1.65rem minmax(0, 1fr);
      align-items: center;
      gap: 0.4rem;
      min-width: 0;
      line-height: 1.2;
    }

    .edge-key {
      position: relative;
      width: 1.65rem;
      height: 0.7rem;
      color: var(--key-color, #718096);
    }

    .edge-key::before {
      position: absolute;
      top: 50%;
      right: 0.22rem;
      left: 0;
      border-top: 2px solid currentColor;
      content: '';
      transform: translateY(-50%);
    }

    .edge-key::after {
      position: absolute;
      top: 50%;
      right: 0;
      width: 0;
      height: 0;
      border-top: 0.22rem solid transparent;
      border-bottom: 0.22rem solid transparent;
      border-left: 0.32rem solid currentColor;
      content: '';
      transform: translateY(-50%);
    }

    .edge-key.low-confidence::before {
      border-top-style: dashed;
    }

    .evidence-glyph-key {
      justify-self: center;
      color: #718096;
      font-size: 0.78rem;
      line-height: 1;
      white-space: nowrap;
    }

    .evidence-glyph-key.attested {
      color: #7053b6;
    }

    .node-shape {
      justify-self: center;
      width: 0.9rem;
      height: 0.65rem;
      border: 1.5px solid var(--node-color);
      background: color-mix(in srgb, var(--node-color) 14%, transparent);
    }

    .node-shape.computation {
      clip-path: polygon(25% 0, 75% 0, 100% 50%, 75% 100%, 25% 100%, 0 50%);
    }

    .node-shape.data {
      border-radius: 999px;
    }

    .node-shape.software {
      clip-path: polygon(
        18% 0,
        82% 0,
        100% 25%,
        100% 75%,
        82% 100%,
        18% 100%,
        0 75%,
        0 25%
      );
    }

    .node-shape.reference {
      clip-path: polygon(0 0, 72% 0, 100% 50%, 72% 100%, 0 100%);
    }

    .node-shape.context {
      border-style: dashed;
      border-radius: 3px;
    }

    .node-shape.output {
      border-width: 2px;
      border-radius: 3px;
      background: color-mix(in srgb, var(--node-color) 28%, transparent);
      box-shadow: 0 0 0 2px
        color-mix(in srgb, var(--node-color) 16%, transparent);
    }

    .focus-key {
      justify-self: center;
      width: 0.8rem;
      height: 0.62rem;
      border: 2px solid var(--focus-color, #0f766e);
      border-radius: 3px;
      background: color-mix(
        in srgb,
        var(--focus-color, #0f766e) 12%,
        transparent
      );
      box-shadow: 0 0 5px
        color-mix(in srgb, var(--focus-color, #0f766e) 70%, transparent);
    }

    .focus-key.inspector-selected-key {
      --focus-color: currentColor;
      box-shadow: 0 0 0 2px color-mix(in srgb, currentColor 32%, transparent);
    }

    .edge-preview {
      position: fixed;
      z-index: 5;
      width: min(19rem, calc(100vw - 1rem));
      padding: 0.65rem 0.75rem;
      border: 1px solid var(--border-color-default, #d8dee4);
      border-radius: 8px;
      background: var(--surface-elevated, #fff);
      box-shadow: 0 12px 28px rgb(15 23 42 / 18%);
      pointer-events: none;
      font-size: 0.75rem;
      line-height: 1.45;
    }

    .preview-title,
    .panel-title {
      color: var(--text-color-primary, #1f2933);
      font-weight: 650;
      overflow-wrap: anywhere;
    }

    .panel-title {
      font-size: 1rem;
      line-height: 1.35;
    }

    .inspector {
      position: fixed;
      top: 0;
      right: 0;
      z-index: 3;
      display: flex;
      flex-direction: column;
      width: min(27rem, 92vw);
      height: 100dvh;
      border-left: 1px solid var(--border-color-default, #d8dee4);
      background: var(--surface-elevated, #fff);
      box-shadow: -12px 0 36px rgb(15 23 42 / 16%);
      animation: panel-in 160ms ease-out;
    }

    .inspector-header {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: 1rem;
      padding: 1rem 1.1rem;
      border-bottom: 1px solid var(--border-color-default, #e2e8f0);
    }

    .inspector-heading {
      display: grid;
      min-width: 0;
      gap: 0.45rem;
    }

    .graph-id {
      color: var(--text-color-muted, #5d6978);
      font-family: var(--code-font-family, ui-monospace, monospace);
      font-size: 0.72rem;
      overflow-wrap: anywhere;
    }

    .badge-row {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.3rem;
      min-width: 0;
    }

    .badge {
      display: inline-flex;
      align-items: center;
      min-height: 1.25rem;
      padding: 0.1rem 0.4rem;
      border-radius: 999px;
      background: color-mix(in srgb, var(--badge-color) 12%, transparent);
      color: var(--badge-color);
      font-size: 0.65rem;
      font-weight: 650;
      line-height: 1.2;
      white-space: nowrap;
    }

    .badge.blue {
      --badge-color: light-dark(#2563a9, #7db8ff);
    }

    .badge.teal {
      --badge-color: light-dark(#0f766e, #47dcc8);
    }

    .badge.violet {
      --badge-color: light-dark(#7053b6, #c4b5fd);
    }

    .badge.amber {
      --badge-color: light-dark(#a16207, #f8c65c);
    }

    .badge.gray {
      --badge-color: light-dark(#5d6978, #a9b5c5);
    }

    .inspector-body {
      display: grid;
      align-content: start;
      padding: 0 1.1rem 1rem;
      overflow-y: auto;
      overscroll-behavior: contain;
    }

    .inspector-group {
      display: grid;
      margin: 0;
      border-top: 1px solid var(--border-color-muted, #e5e9ee);
      padding: 1rem 0 0.15rem;
    }

    .section-title {
      margin: 0 0 0.55rem;
      color: var(--text-color-muted, #5d6978);
      font-size: 0.7rem;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.055em;
    }

    .disclosure {
      border-top: 1px solid
        color-mix(in srgb, var(--border-color-muted, #e5e9ee) 75%, transparent);
    }

    .section-title + .disclosure,
    .section-title + .relationship-list .relationship-row:first-child {
      border-top: 0;
    }

    .disclosure > summary {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: center;
      gap: 0.65rem;
      min-height: 2.75rem;
      padding: 0.45rem 0.1rem;
      border-radius: 4px;
      cursor: pointer;
      list-style: none;
    }

    .disclosure > summary::-webkit-details-marker {
      display: none;
    }

    .disclosure > summary:hover {
      background: color-mix(
        in srgb,
        var(--text-color-muted, #64748b) 6%,
        transparent
      );
    }

    .disclosure > summary:focus-visible {
      outline: 2px solid var(--color-accent, #0f766e);
      outline-offset: 2px;
    }

    .summary-content {
      display: grid;
      gap: 0.3rem;
      min-width: 0;
    }

    .summary-label {
      font-size: 0.78rem;
      font-weight: 620;
      line-height: 1.35;
      overflow-wrap: anywhere;
    }

    .summary-location {
      color: var(--text-color-muted, #5d6978);
      font-size: 0.68rem;
      overflow-wrap: anywhere;
    }

    .caret {
      width: 0.5rem;
      height: 0.5rem;
      margin-right: 0.35rem;
      border-right: 1.5px solid currentColor;
      border-bottom: 1.5px solid currentColor;
      color: var(--text-color-muted, #5d6978);
      transform: rotate(45deg);
      transition: transform 120ms ease;
    }

    .disclosure[open] > summary .caret {
      transform: rotate(225deg);
    }

    .record-grid {
      display: grid;
      grid-template-columns: minmax(5rem, 6.25rem) minmax(0, 1fr);
      gap: 0.35rem 0.7rem;
      margin: 0;
      padding: 0.15rem 0.1rem 0.85rem;
      font-size: 0.75rem;
    }

    .record-grid dt {
      color: var(--text-color-muted, #5d6978);
    }

    .record-grid dd {
      min-width: 0;
      margin: 0;
      overflow-wrap: anywhere;
    }

    .metadata-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
      gap: 0.15rem;
      min-width: 0;
      margin: 0;
      padding: 0;
    }

    .metadata-grid.metadata-depth-1 {
      padding-left: 0.65rem;
      border-left: 1px solid
        color-mix(in srgb, var(--border-color-muted, #e5e9ee) 80%, transparent);
    }

    .metadata-grid > dt {
      color: var(--text-color-muted, #5d6978);
      font-size: 0.68rem;
    }

    .metadata-grid > dt:not(:first-child) {
      margin-top: 0.4rem;
    }

    .metadata-grid > dd {
      min-width: 0;
      margin: 0;
      overflow-wrap: anywhere;
    }

    .metadata-object,
    .metadata-collection {
      min-width: 0;
    }

    .metadata-object {
      display: grid;
      gap: 0.35rem;
    }

    .metadata-object-heading {
      display: flex;
      flex-wrap: wrap;
      gap: 0.3rem;
    }

    .metadata-collection {
      display: grid;
      gap: 0.55rem;
    }

    .metadata-item {
      display: grid;
      gap: 0.35rem;
      min-width: 0;
      padding-top: 0.55rem;
      border-top: 1px solid
        color-mix(in srgb, var(--border-color-muted, #e5e9ee) 75%, transparent);
    }

    .metadata-item:first-child {
      padding-top: 0;
      border-top: 0;
    }

    .metadata-item-heading {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.35rem;
      min-width: 0;
      font-weight: 620;
      overflow-wrap: anywhere;
    }

    .metadata-type {
      display: inline-flex;
      max-width: 100%;
      padding: 0.08rem 0.34rem;
      border-radius: 999px;
      background: color-mix(
        in srgb,
        var(--text-color-muted, #64748b) 9%,
        transparent
      );
      color: var(--text-color-secondary, #475569);
      font-size: 0.65rem;
      font-weight: 600;
      line-height: 1.3;
      overflow-wrap: anywhere;
    }

    .metadata-tokens {
      display: inline-flex;
      flex-wrap: wrap;
      gap: 0.25rem;
      min-width: 0;
    }

    .metadata-token {
      display: inline-flex;
      min-width: 0;
      max-width: 100%;
      padding: 0.08rem 0.32rem;
      border-radius: 4px;
      background: color-mix(
        in srgb,
        var(--text-color-muted, #64748b) 9%,
        transparent
      );
    }

    .metadata-null {
      color: var(--text-color-muted, #5d6978);
      font-style: italic;
    }

    .metadata-link {
      color: light-dark(#2563a9, #7db8ff);
      text-decoration-thickness: 1px;
      text-underline-offset: 0.12em;
      overflow-wrap: anywhere;
    }

    .metadata-link:focus-visible {
      border-radius: 2px;
      outline: 2px solid var(--color-accent, #0f766e);
      outline-offset: 2px;
    }

    .metadata-identifier,
    .metadata-json {
      display: inline-block;
      min-width: 0;
      max-width: 100%;
      color: inherit;
      font-family: var(--code-font-family, ui-monospace, monospace);
      font-size: 0.68rem;
      vertical-align: bottom;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .empty-state {
      margin: 0;
      padding: 0.3rem 0 0.85rem;
      color: var(--text-color-muted, #5d6978);
      font-size: 0.75rem;
    }

    .relationship-list {
      display: grid;
    }

    .relationship-row {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr) auto;
      align-items: center;
      gap: 0.5rem;
      min-height: 2.5rem;
      border-top: 1px solid
        color-mix(in srgb, var(--border-color-muted, #e5e9ee) 75%, transparent);
      font-size: 0.75rem;
    }

    .relationship-endpoints {
      min-width: 0;
      overflow-wrap: anywhere;
    }

    @keyframes panel-in {
      from {
        transform: translateX(1rem);
        opacity: 0;
      }
    }

    @media (max-width: 760px) {
      .controls,
      .controls.inspector-visible {
        top: 0.5rem;
        right: 0.5rem;
      }

      .settings-card {
        width: calc(100vw - 1rem);
      }

      select {
        width: 100%;
      }

      .legend {
        top: 0.5rem;
        left: 0.5rem;
        width: fit-content;
        max-width: calc(100vw - 1rem);
        max-height: min(65dvh, 34rem);
        overflow: hidden;
      }

      .legend[open] {
        width: calc(100vw - 1rem);
      }

      .legend-groups {
        max-height: calc(min(65dvh, 34rem) - 2.35rem);
      }

      .inspector {
        top: auto;
        bottom: 0;
        width: 100vw;
        height: min(72dvh, 38rem);
        border-top: 1px solid var(--border-color-default, #d8dee4);
        border-left: 0;
        border-radius: 14px 14px 0 0;
        animation-name: sheet-in;
      }

      @keyframes sheet-in {
        from {
          transform: translateY(1rem);
          opacity: 0;
        }
      }
    }

    @media (prefers-reduced-motion: reduce) {
      *,
      *::before,
      *::after {
        animation-duration: 0.01ms !important;
        transition-duration: 0.01ms !important;
      }
    }
  `

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener(
      'stencila-color-scheme-changed',
      this.onColorSchemeChange
    )
    window.addEventListener('keydown', this.onKeydown)
    this.syncColorScheme()
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
      changed.has('layoutSpacing') ||
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
    window.removeEventListener('keydown', this.onKeydown)
    window.clearTimeout(this.hoverTimer)
    this.cy?.destroy()
    this.cy = undefined
    super.disconnectedCallback()
  }

  override render() {
    if (this.error) {
      return html`<div class="message">${this.error}</div>`
    }

    return html`
      <div class="canvas"></div>
      <div
        class=${`controls ${
          this.pinnedEdgeId || this.selectedNodeId ? 'inspector-visible' : ''
        }`}
        @click=${this.stopClickPropagation}
      >
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
        ${this.settingsOpen ? this.renderSettings() : null}
      </div>
      ${this.renderLegend()} ${this.renderEdgePreview()}
      ${this.renderInspector()}
    `
  }

  private renderSettings() {
    return html`
      <div class="settings-card">
        ${this.renderSelect(
          'Projection',
          this.projection,
          [
            ['auto', 'Auto'],
            ['full', 'Full'],
            ['data-flow', 'Data flow'],
            ['software-dependencies', 'Software dependencies'],
            ['citations', 'Citations'],
            ['reactivity', 'Reactivity'],
          ],
          this.onProjectionChange
        )}
        ${this.renderSelect(
          'Layout',
          this.layout,
          [
            ['breadthfirst', 'Lineage'],
            ['cose', 'Force'],
            ['grid', 'Grid'],
            ['circle', 'Circle'],
          ],
          this.onLayoutChange
        )}
        <label class="setting range-setting">
          <span class="setting-label">Spacing</span>
          <output for="graph-spacing">${this.layoutSpacing}</output>
          <input
            id="graph-spacing"
            type="range"
            min="0"
            max=${layoutSpacings.length - 1}
            step="1"
            .value=${String(layoutSpacings.indexOf(this.layoutSpacing))}
            aria-label="Graph spacing"
            aria-valuetext=${this.layoutSpacing}
            @input=${this.onLayoutSpacingChange}
          />
        </label>
        ${this.renderSelect(
          'Detail',
          this.detail,
          [
            ['low', 'Low'],
            ['medium', 'Medium'],
            ['high', 'High'],
          ],
          this.onDetailChange
        )}
        ${this.renderCheckbox(
          'Structure',
          this.effectiveIncludeStructureEdges(),
          this.onStructureChange
        )}
        ${this.renderCheckbox(
          'Low confidence',
          this.includeLowConfidenceEdges,
          this.onConfidenceChange
        )}
        ${this.renderCheckbox(
          'Collapse citations',
          this.collapseCitationNodes,
          this.onCitationCollapseChange
        )}
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
          ${this.resolvedProjection} · ${this.nodeCount} nodes ·
          ${this.edgeCount} edges
        </span>
      </div>
    `
  }

  private renderSelect(
    label: string,
    value: string,
    options: [string, string][],
    onChange: (event: Event) => void
  ) {
    return html`
      <label class="setting">
        <span class="setting-label">${label}</span>
        <select .value=${value} @change=${onChange}>
          ${options.map(
            ([optionValue, optionLabel]) =>
              html`<option value=${optionValue}>${optionLabel}</option>`
          )}
        </select>
      </label>
    `
  }

  private renderCheckbox(
    label: string,
    checked: boolean,
    onChange: (event: Event) => void
  ) {
    return html`
      <label class="checkbox-setting">
        <input type="checkbox" .checked=${checked} @change=${onChange} />
        ${label}
      </label>
    `
  }

  private renderLegend() {
    return html`
      <details
        class="legend"
        .open=${this.legendOpen}
        @toggle=${this.onLegendToggle}
      >
        <summary>
          <span class="legend-title">Legend</span>
          <span class="legend-chevron" aria-hidden="true"></span>
        </summary>
        <div class="legend-groups">
          <section class="legend-group">
            <h2 class="legend-group-title">Relationships</h2>
            <ul class="legend-list">
              ${this.relationshipLegendItem('#2563a9', 'Read or include')}
              ${this.relationshipLegendItem('#0f766e', 'Derive or convert')}
              ${this.relationshipLegendItem('#7053b6', 'Generate or write')}
              ${this.relationshipLegendItem('#a16207', 'Software dependency')}
              ${this.relationshipLegendItem('#718096', 'Structure or containment')}
            </ul>
          </section>
          <section class="legend-group">
            <h2 class="legend-group-title">Nodes</h2>
            <ul class="legend-list">
              ${this.nodeLegendItem('artifact', '#2563a9', 'File or document')}
              ${this.nodeLegendItem('computation', '#0f766e', 'Code or function')}
              ${this.nodeLegendItem('data', '#0f766e', 'Data or symbol')}
              ${this.nodeLegendItem('software', '#a16207', 'Package or environment')}
              ${this.nodeLegendItem('reference', '#7053b6', 'Reference or citation')}
              ${this.nodeLegendItem('context', '#718096', 'Context or container')}
              ${this.nodeLegendItem('output', '#7053b6', 'Generated output')}
            </ul>
          </section>
          <section class="legend-group">
            <h2 class="legend-group-title">Evidence & selection</h2>
            <ul class="legend-list">
              <li class="legend-item">
                <span class="evidence-glyph-key" aria-hidden="true">●</span>
                Evidence recorded
              </li>
              <li class="legend-item">
                <span
                  class="evidence-glyph-key attested"
                  aria-hidden="true"
                >◆</span>
                Attested evidence
              </li>
              <li class="legend-item">
                <span
                  class="evidence-glyph-key"
                  aria-hidden="true"
                >●●●</span>
                Multiple evidence items
              </li>
              <li class="legend-item">
                <span
                  class="edge-key low-confidence"
                  aria-hidden="true"
                ></span>
                Lower confidence
              </li>
              <li class="legend-item">
                <span class="focus-key" aria-hidden="true"></span>
                In dependency trace
              </li>
              <li class="legend-item">
                <span
                  class="focus-key inspector-selected-key"
                  aria-hidden="true"
                ></span>
                Shown in inspector
              </li>
            </ul>
          </section>
        </div>
      </details>
    `
  }

  private relationshipLegendItem(color: string, label: string) {
    return html`
      <li class="legend-item">
        <span
          class="edge-key"
          style=${`--key-color:${color}`}
          aria-hidden="true"
        ></span>
        ${label}
      </li>
    `
  }

  private nodeLegendItem(shape: string, color: string, label: string) {
    return html`
      <li class="legend-item">
        <span
          class=${`node-shape ${shape}`}
          style=${`--node-color:${color}`}
          aria-hidden="true"
        ></span>
        ${label}
      </li>
    `
  }

  private renderEdgePreview() {
    const edge = this.hoveredEdgeId
      ? this.edgesById.get(this.hoveredEdgeId)
      : undefined
    if (!edge || edge.id === this.pinnedEdgeId) {
      return null
    }

    const evidence = presentEvidence(edge)
    const locationOrDescription =
      evidence[0]?.location ?? evidence[0]?.evidence.description

    return html`
      <div class="edge-preview" role="tooltip">
        <div class="preview-title">${this.edgeTitle(edge)}</div>
        <div>
          ${evidence.length
            ? evidence
                .map(
                  (item) =>
                    `${item.evidence.kind} (${item.confidence.toLowerCase()})`
                )
                .join(', ')
            : 'No recorded evidence'}
        </div>
        ${locationOrDescription
          ? html`<div class="muted">${locationOrDescription}</div>`
          : null}
        <div class="muted">${edgeSummary(edge)}</div>
      </div>
    `
  }

  private renderInspector() {
    const edge = this.pinnedEdgeId
      ? this.edgesById.get(this.pinnedEdgeId)
      : undefined
    const node = this.selectedNodeId
      ? this.view?.nodes.find((item) => item.id === this.selectedNodeId)
      : undefined
    if (!edge && !node) {
      return null
    }

    return html`
      <aside
        class="inspector"
        role="complementary"
        aria-label="Graph inspector"
        @click=${this.stopClickPropagation}
      >
        <header class="inspector-header">
          <div class="inspector-heading">
            <div class="panel-title">
              ${edge ? this.edgeEndpointsTitle(edge) : node?.label}
            </div>
            ${edge
              ? html`
                  <div class="badge-row">
                    ${this.renderBadge(
                      edge.label,
                      relationshipBadgeTone(edge.kind)
                    )}
                    <span class="muted">${edgeSummary(edge)}</span>
                  </div>
                `
              : html`
                  <div class="badge-row">
                    ${this.renderBadge(
                      humanizeLabel((node as GraphViewNode).kind),
                      nodeBadgeTone((node as GraphViewNode).kind)
                    )}
                    ${this.renderBadge(
                      this.nodeSchemaType(node as GraphViewNode),
                      'gray'
                    )}
                  </div>
                  <div class="graph-id">${node?.id}</div>
                `}
          </div>
          <button
            title="Close"
            aria-label="Close graph inspector"
            @click=${this.closeInspector}
          >
            <span class="i-lucide:x"></span>
          </button>
        </header>
        <div class="inspector-body">
          ${keyed(
            edge ? `edge:${edge.id}` : `node:${node?.id}`,
            edge
              ? this.renderEdgeInspector(edge)
              : this.renderNodeInspector(node as GraphViewNode)
          )}
        </div>
      </aside>
    `
  }

  private renderEdgeInspector(edge: GraphViewEdge) {
    const evidence = presentEvidence(edge)
    const actions = presentActions(edge)

    return html`
      <section class="inspector-group evidence-group">
        <h2 class="section-title">Evidence (${evidence.length})</h2>
        ${evidence.length
          ? evidence.map((item) =>
              this.renderEvidence(item, edge.count > 1)
            )
          : html`<p class="empty-state">No recorded evidence</p>`}
      </section>
      <section class="inspector-group activity-group">
        <h2 class="section-title">
          Associated activities (${actions.length})
        </h2>
        ${actions.length
          ? actions.map((action) => this.renderAction(action, edge.count > 1))
          : html`<p class="empty-state">No associated activities</p>`}
      </section>
      ${edge.count > 1
        ? html`
            <section class="inspector-group relationship-group">
              <h2 class="section-title">
                Contributing relationships (${edge.count})
              </h2>
              <div class="relationship-list">
                ${edge.edges.map(
                  (rawEdge, index) => html`
                    <div class="relationship-row">
                      ${this.renderBadge(`#${index + 1}`, 'gray')}
                      <span class="relationship-endpoints">
                        ${this.edgeEndpoints(rawEdge.source, rawEdge.target)}
                      </span>
                      ${this.renderBadge(
                        humanizeLabel(rawEdge.kind),
                        relationshipBadgeTone(rawEdge.kind)
                      )}
                    </div>
                  `
                )}
              </div>
            </section>
          `
        : null}
    `
  }

  private renderNodeInspector(node: GraphViewNode) {
    const properties = nodeProperties(node)

    return html`
      <section class="inspector-group node-properties">
        <details class="disclosure properties-disclosure" .open=${true}>
          <summary>
            <span class="summary-content">
              <span class="summary-label">Properties</span>
              <span class="muted">${properties.length} recorded</span>
            </span>
            <span class="caret" aria-hidden="true"></span>
          </summary>
          ${properties.length
            ? this.renderRecordGrid(properties)
            : html`<p class="empty-state">No additional properties</p>`}
        </details>
      </section>
    `
  }

  private renderEvidence(
    item: ReturnType<typeof presentEvidence>[number],
    aggregated: boolean
  ) {
    const label = evidenceLabel(item)

    return html`
      <details class="disclosure evidence-disclosure" .open=${true}>
        <summary>
          <span class="summary-content">
            <span class="summary-label">${label}</span>
            <span class="badge-row">
              ${this.renderBadge(
                humanizeLabel(item.evidence.kind),
                evidenceBadgeTone(item.evidence.kind)
              )}
              ${this.renderBadge(
                item.confidence,
                confidenceBadgeTone(item.confidence)
              )}
              ${aggregated
                ? this.renderBadge(`#${item.contributor}`, 'gray')
                : null}
            </span>
            ${item.location && item.location !== label
              ? html`<span class="summary-location">${item.location}</span>`
              : null}
          </span>
          <span class="caret" aria-hidden="true"></span>
        </summary>
        ${this.renderRecordGrid([
          ...(item.evidence.description
            ? [['Description', item.evidence.description] as PresentedDetail]
            : []),
          ...(item.location
            ? [['Location', item.location] as PresentedDetail]
            : []),
          ...(item.source !== undefined
            ? [['Source', item.source] as PresentedDetail]
            : []),
          ...(item.evidence.recordedAt
            ? [
                [
                  'Recorded',
                  item.evidence.recordedAt,
                ] as PresentedDetail,
              ]
            : []),
          ...item.details,
        ])}
      </details>
    `
  }

  private renderAction(
    item: ReturnType<typeof presentActions>[number],
    aggregated: boolean
  ) {
    const badge = actionBadge(item.action.type)

    return html`
      <details class="disclosure activity-disclosure" .open=${true}>
        <summary>
          <span class="summary-content">
            <span class="summary-label">${actionLabel(item)}</span>
            <span class="badge-row">
              ${this.renderBadge(badge.label, badge.tone)}
              ${aggregated
                ? this.renderBadge(`#${item.contributor}`, 'gray')
                : null}
            </span>
            <span class="summary-location">
              ${this.edgeEndpoints(
                item.graphEdge.source,
                item.graphEdge.target
              )}
            </span>
          </span>
          <span class="caret" aria-hidden="true"></span>
        </summary>
        ${this.renderRecordGrid(item.details)}
      </details>
    `
  }

  private renderRecordGrid(
    details: PresentedDetail[],
    depth = 0,
    nested = false
  ) {
    const depthClass = `metadata-depth-${Math.min(depth, 2)}`
    return html`
      <dl class=${nested ? `metadata-grid ${depthClass}` : 'record-grid'}>
        ${details.map(
          ([key, value]) =>
            html`<dt>${humanizeLabel(key)}</dt>
              <dd>${this.renderMetadataValue(value, depth)}</dd>`
        )}
      </dl>
    `
  }

  private renderMetadataValue(value: unknown, depth: number): unknown {
    if (value === null) {
      return html`<span class="metadata-null" title="No value">—</span>`
    }

    if (typeof value === 'string') {
      const urlLabel = webUrlLabel(value)
      if (urlLabel) {
        return html`
          <a
            class="metadata-link"
            href=${value}
            target="_blank"
            rel="noreferrer"
            title=${value}
          >${urlLabel}<span aria-hidden="true"> ↗</span></a>
        `
      }

      return isLongMetadataValue(value)
        ? html`<code class="metadata-identifier" title=${value}>${value}</code>`
        : value
    }

    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value)
    }

    if (depth >= 4) {
      const json = formatValue(value)
      return html`<code class="metadata-json" title=${json}>${json}</code>`
    }

    if (Array.isArray(value)) {
      return this.renderMetadataArray(value, depth)
    }

    if (isRecord(value)) {
      const typeLabel = metadataTypeLabel(value)
      const entries = sortedDetails(value, []).filter(
        ([key]) => key !== 'type'
      )
      if (!typeLabel && entries.length === 0) {
        return html`<span class="metadata-null">Empty</span>`
      }

      return html`
        <div class="metadata-object">
          ${typeLabel
            ? html`<div class="metadata-object-heading">
                <span class="metadata-type">${typeLabel}</span>
              </div>`
            : null}
          ${entries.length
            ? this.renderRecordGrid(entries, depth + 1, true)
            : null}
        </div>
      `
    }

    return formatValue(value)
  }

  private renderMetadataArray(values: unknown[], depth: number) {
    if (values.length === 0) {
      return html`<span class="metadata-null">None</span>`
    }

    if (values.every((value) => !isRecord(value) && !Array.isArray(value))) {
      return html`
        <span class="metadata-tokens">
          ${values.map(
            (value) => html`
              <span class="metadata-token">
                ${this.renderMetadataValue(value, depth + 1)}
              </span>
            `
          )}
        </span>
      `
    }

    return html`
      <div class="metadata-collection">
        ${values.map((value, index) => {
          if (!isRecord(value)) {
            return html`
              <div class="metadata-item">
                <div class="metadata-item-heading">Item ${index + 1}</div>
                ${this.renderMetadataValue(value, depth + 1)}
              </div>
            `
          }

          const identity = metadataIdentity(value)
          const typeLabel = metadataTypeLabel(value)
          const heading = identity
            ? identity.key === 'type' || identity.key === 'kind'
              ? humanizeLabel(identity.label)
              : identity.label
            : `Item ${index + 1}`
          const entries = sortedDetails(value, []).filter(
            ([key]) => key !== identity?.key && key !== 'type'
          )
          return html`
            <div class="metadata-item">
              <div class="metadata-item-heading">
                <span>${heading}</span>
                ${typeLabel && identity?.key !== 'type'
                  ? html`<span class="metadata-type">${typeLabel}</span>`
                  : null}
              </div>
              ${entries.length
                ? this.renderRecordGrid(entries, depth + 1, true)
                : null}
            </div>
          `
        })}
      </div>
    `
  }

  private renderBadge(label: string, tone: BadgeTone) {
    return html`<span class=${`badge ${tone}`}>${label}</span>`
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
      this.error =
        error instanceof Error ? error.message : 'Unable to parse graph data.'
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

    this.view = view
    this.edgesById = new Map(view.edges.map((edge) => [edge.id, edge]))
    this.nodeCount = view.nodes.length
    this.edgeCount = view.edges.length
    this.resolvedProjection = view.preset

    if (this.pinnedEdgeId && !this.edgesById.has(this.pinnedEdgeId)) {
      this.pinnedEdgeId = undefined
    }
    if (
      this.selectedNodeId &&
      !view.nodes.some((node) => node.id === this.selectedNodeId)
    ) {
      this.selectedNodeId = undefined
    }
    if (
      this.tracedNodeId &&
      !view.nodes.some((node) => node.id === this.tracedNodeId)
    ) {
      this.tracedNodeId = undefined
    }

    const theme = {
      ...buildCytoscapeTheme(this),
      fontFamily:
        getComputedStyle(this).fontFamily || 'Inter, system-ui, sans-serif',
    }
    this.cy = cytoscape(
      toCytoscapeOptions(
        view,
        this.canvas,
        this.layout,
        this.layoutSpacing,
        theme
      )
    )
    this.bindGraphEvents()
    this.applyLineage()
    this.applyInspectorSelection()
  }

  private bindGraphEvents() {
    if (!this.cy) {
      return
    }

    this.cy.on('mouseover', 'edge', this.onEdgeOver)
    this.cy.on('mousemove', 'edge', this.onEdgeMove)
    this.cy.on('mouseout', 'edge', this.onEdgeOut)
    this.cy.on('tap', 'edge', this.onEdgeTap)
    this.cy.on('tap', 'node', this.onNodeTap)
    this.cy.on('tap', this.onCanvasTap)
  }

  private onEdgeOver = (event: EventObject) => {
    const edge = event.target as EdgeSingular
    this.updateHoverPoint(event)
    window.clearTimeout(this.hoverTimer)
    this.hoverTimer = window.setTimeout(() => {
      this.hoveredEdgeId = edge.id()
      void this.updateComplete.then(() => this.positionEdgePreview())
    }, 150)
  }

  private onEdgeMove = (event: EventObject) => {
    this.updateHoverPoint(event)
    if (this.hoveredEdgeId) {
      void this.positionEdgePreview()
    }
  }

  private onEdgeOut = () => {
    window.clearTimeout(this.hoverTimer)
    this.hoveredEdgeId = undefined
  }

  private onEdgeTap = (event: EventObject) => {
    event.stopPropagation()
    const edge = event.target as EdgeSingular
    this.pinnedEdgeId = edge.id()
    this.selectedNodeId = undefined
    this.hoveredEdgeId = undefined
    this.applyInspectorSelection()
  }

  private onNodeTap = (event: EventObject) => {
    event.stopPropagation()
    const node = event.target as NodeSingular
    this.selectedNodeId = node.id()
    this.pinnedEdgeId = undefined
    this.tracedNodeId =
      this.tracedNodeId === node.id() ? undefined : node.id()
    this.applyLineage()
    this.applyInspectorSelection()
  }

  private onCanvasTap = (event: EventObject) => {
    if (event.target !== this.cy) {
      return
    }

    this.settingsOpen = false
    this.tracedNodeId = undefined
    this.applyLineage()
  }

  private applyLineage() {
    if (!this.cy || !this.view) {
      return
    }

    this.cy.batch(() => {
      this.cy
        ?.elements()
        .removeClass(
          'lineage-dim lineage-upstream-node lineage-downstream-node lineage-overlap-node lineage-upstream-edge lineage-downstream-edge lineage-overlap-edge lineage-selected'
        )
      if (!this.tracedNodeId) {
        return
      }

      const neighborhood = dependencyNeighborhood(
        this.view as GraphView,
        this.tracedNodeId
      )
      this.cy?.elements().addClass('lineage-dim')
      for (const nodeId of neighborhood.upstreamNodeIds) {
        this.cy?.getElementById(nodeId).addClass('lineage-upstream-node')
      }
      for (const nodeId of neighborhood.downstreamNodeIds) {
        this.cy?.getElementById(nodeId).addClass('lineage-downstream-node')
      }
      for (const nodeId of neighborhood.overlapNodeIds) {
        this.cy?.getElementById(nodeId).addClass('lineage-overlap-node')
      }
      for (const edgeId of neighborhood.upstreamEdgeIds) {
        this.cy?.getElementById(edgeId).addClass('lineage-upstream-edge')
      }
      for (const edgeId of neighborhood.downstreamEdgeIds) {
        this.cy?.getElementById(edgeId).addClass('lineage-downstream-edge')
      }
      for (const edgeId of neighborhood.overlapEdgeIds) {
        this.cy?.getElementById(edgeId).addClass('lineage-overlap-edge')
      }
      this.cy
        ?.getElementById(this.tracedNodeId)
        .addClass('lineage-selected')
    })
  }

  private applyInspectorSelection() {
    if (!this.cy) {
      return
    }

    this.cy.batch(() => {
      this.cy
        ?.elements()
        .removeClass('inspector-selected-node inspector-selected-edge')
      if (this.pinnedEdgeId) {
        this.cy
          ?.getElementById(this.pinnedEdgeId)
          .addClass('inspector-selected-edge')
      }
      if (this.selectedNodeId) {
        this.cy
          ?.getElementById(this.selectedNodeId)
          .addClass('inspector-selected-node')
      }
    })
  }

  private updateHoverPoint(event: EventObject) {
    if (!this.canvas) {
      return
    }

    const rect = this.canvas.getBoundingClientRect()
    this.hoverPoint = {
      x: rect.left + event.renderedPosition.x,
      y: rect.top + event.renderedPosition.y,
    }
  }

  private async positionEdgePreview() {
    if (!this.edgePreview) {
      return
    }

    const reference: VirtualElement = {
      getBoundingClientRect: () =>
        new DOMRect(this.hoverPoint.x, this.hoverPoint.y, 0, 0),
    }
    const position = await computePosition(reference, this.edgePreview, {
      placement: 'top',
      middleware: [offset(10), flip(), shift({ padding: 8 })],
    })
    Object.assign(this.edgePreview.style, {
      left: `${position.x}px`,
      top: `${position.y}px`,
    })
  }

  private edgeTitle(edge: GraphViewEdge) {
    return `${this.edgeEndpointsTitle(edge)} · ${edge.label}`
  }

  private edgeEndpointsTitle(edge: GraphViewEdge) {
    return this.edgeEndpoints(edge.source, edge.target)
  }

  private edgeEndpoints(sourceId: string, targetId: string) {
    const source =
      this.view?.nodes.find((node) => node.id === sourceId)?.label ?? sourceId
    const target =
      this.view?.nodes.find((node) => node.id === targetId)?.label ?? targetId
    return `${source} → ${target}`
  }

  private nodeSchemaType(node: GraphViewNode): string {
    const value = node.node.node
    if (typeof value === 'object' && value !== null && 'type' in value) {
      return String(value.type)
    }

    return typeof value
  }

  private fit = () => this.cy?.fit(undefined, 40)
  private onColorSchemeChange = () => {
    this.syncColorScheme()
    this.renderGraph()
  }
  private syncColorScheme() {
    const scheme = document.documentElement.getAttribute('data-color-scheme')
    if (scheme === 'light' || scheme === 'dark') {
      this.dataset.colorScheme = scheme
    }
  }
  private toggleSettings = () => (this.settingsOpen = !this.settingsOpen)
  private stopClickPropagation = (event: Event) => event.stopPropagation()
  private closeInspector = (): void => {
    this.pinnedEdgeId = undefined
    this.selectedNodeId = undefined
    this.applyInspectorSelection()
  }

  private onKeydown = (event: KeyboardEvent) => {
    if (event.key !== 'Escape') {
      return
    }
    this.settingsOpen = false
    this.pinnedEdgeId = undefined
    this.selectedNodeId = undefined
    this.tracedNodeId = undefined
    this.applyLineage()
    this.applyInspectorSelection()
  }

  private onLegendToggle = (event: Event) => {
    this.legendOpen = (event.currentTarget as HTMLDetailsElement).open
  }

  private onProjectionChange = (event: Event) => {
    this.projection = (event.currentTarget as HTMLSelectElement)
      .value as GraphViewPreset
    this.includeStructureEdges = undefined
  }

  private onLayoutChange = (event: Event) => {
    this.layout = (event.currentTarget as HTMLSelectElement).value as GraphLayout
  }

  private onLayoutSpacingChange = (event: Event) => {
    const index = Number((event.currentTarget as HTMLInputElement).value)
    this.layoutSpacing = layoutSpacings[index] ?? 'cozy'
  }

  private onDetailChange = (event: Event) => {
    this.detail = (event.currentTarget as HTMLSelectElement)
      .value as GraphProjectionDetail
  }

  private onStructureChange = (event: Event) => {
    this.includeStructureEdges = (event.currentTarget as HTMLInputElement).checked
  }

  private onConfidenceChange = (event: Event) => {
    this.includeLowConfidenceEdges = (
      event.currentTarget as HTMLInputElement
    ).checked
  }

  private onCitationCollapseChange = (event: Event) => {
    this.collapseCitationNodes = (
      event.currentTarget as HTMLInputElement
    ).checked
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
