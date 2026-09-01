/** Advisory relation editor for native ResearchObject wrappers. */
import type { ResearchObjectRelationKind } from '@stencila/types'
import { LitElement, html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import type {
  EditNodeRelationDraft,
  ResearchObjectTargetOption,
} from '../nodes/node-properties'
import {
  isRelationKindRecommended,
  relationKindLabel,
  relationKindRange,
  relationKindsForSource,
} from '../nodes/node-properties'
import type { ResearchObjectNodeName } from '../../../tiptap/research-objects'
import { EDIT_PROPERTY_VALUE_CHANGE_EVENT } from './events'

const EXTERNAL_TARGET_VALUE = '__external__'

@customElement('stencila-edit-relations-property')
export class EditRelationsProperty extends LitElement {
  @property({ attribute: false })
  rows: EditNodeRelationDraft[] = []

  @property({ attribute: false })
  source: ResearchObjectNodeName = 'claim'

  @property({ attribute: false })
  targets: ResearchObjectTargetOption[] = []

  @property()
  error?: string

  @state()
  private externalRows = new Set<number>()

  protected override createRenderRoot() {
    return this
  }

  private emitRows(rows: EditNodeRelationDraft[]) {
    this.dispatchEvent(
      new CustomEvent(EDIT_PROPERTY_VALUE_CHANGE_EVENT, {
        bubbles: true,
        composed: true,
        detail: { value: rows },
      })
    )
  }

  private currentOption(
    row: EditNodeRelationDraft
  ): ResearchObjectTargetOption | undefined {
    if (row.targetPos !== undefined) {
      return this.targets.find((option) => option.pos === row.targetPos)
    }

    const target = row.target.startsWith('#')
      ? row.target.slice(1)
      : row.target
    return this.targets.find((option) => option.id === target)
  }

  private isExternal(row: EditNodeRelationDraft, index: number): boolean {
    return (
      this.externalRows.has(index) ||
      (!!row.target && row.targetPos === undefined && !this.currentOption(row))
    )
  }

  private setExternal(index: number, external: boolean) {
    const rows = new Set(this.externalRows)
    if (external) {
      rows.add(index)
    } else {
      rows.delete(index)
    }
    this.externalRows = rows
  }

  private updateKind(index: number, event: Event) {
    const kind = (event.currentTarget as HTMLSelectElement)
      .value as ResearchObjectRelationKind
    this.emitRows(
      this.rows.map((row, rowIndex) =>
        rowIndex === index ? { ...row, kind } : row
      )
    )
  }

  private updateTarget(index: number, event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value
    this.setExternal(index, value === EXTERNAL_TARGET_VALUE)

    this.emitRows(
      this.rows.map((row, rowIndex) => {
        if (rowIndex !== index) {
          return row
        }
        if (value === EXTERNAL_TARGET_VALUE || !value.startsWith('pos:')) {
          return { ...row, target: '', targetPos: undefined }
        }

        const pos = Number(value.slice(4))
        const option = this.targets.find((target) => target.pos === pos)
        return option?.id
          ? { ...row, target: `#${option.id}`, targetPos: undefined }
          : { ...row, target: '', targetPos: pos }
      })
    )
  }

  private updateExternalTarget(index: number, event: Event) {
    const target = (event.currentTarget as HTMLInputElement).value
    this.emitRows(
      this.rows.map((row, rowIndex) =>
        rowIndex === index ? { ...row, target, targetPos: undefined } : row
      )
    )
  }

  private removeRow(index: number) {
    const externalRows = new Set<number>()
    for (const externalIndex of this.externalRows) {
      if (externalIndex < index) {
        externalRows.add(externalIndex)
      } else if (externalIndex > index) {
        externalRows.add(externalIndex - 1)
      }
    }
    this.externalRows = externalRows
    this.emitRows(this.rows.filter((_, rowIndex) => rowIndex !== index))
  }

  private addRow() {
    const [kind] = relationKindsForSource(this.source)
    if (kind) {
      this.emitRows([...this.rows, { kind, target: '' }])
    }
  }

  private optionLabel(option: ResearchObjectTargetOption): string {
    const type = option.typeName.replace(/^./, (letter) => letter.toUpperCase())
    const descriptor = option.label ?? option.title ?? option.excerpt
    const id = option.id ? `#${option.id}` : 'id will be created'
    return descriptor ? `${type} — ${descriptor} (${id})` : `${type} (${id})`
  }

  private renderKindOptions(row: EditNodeRelationDraft) {
    const kinds = relationKindsForSource(this.source)
    const recommended = kinds.filter((kind) =>
      isRelationKindRecommended(this.source, kind)
    )
    const others = kinds.filter(
      (kind) => !isRelationKindRecommended(this.source, kind)
    )
    const render = (kind: ResearchObjectRelationKind) => html`
      <option value=${kind} ?selected=${kind === row.kind}>
        ${relationKindLabel(kind)}
      </option>
    `

    return html`
      ${recommended.length
        ? html`<optgroup label="Recommended">
            ${recommended.map(render)}
          </optgroup>`
        : nothing}
      <optgroup label="Other">${others.map(render)}</optgroup>
    `
  }

  private renderTargetOptions(row: EditNodeRelationDraft) {
    const recommendedTypes = relationKindRange(row.kind)
    const recommended = this.targets.filter((target) =>
      recommendedTypes.includes(target.typeName)
    )
    const others = this.targets.filter(
      (target) => !recommendedTypes.includes(target.typeName)
    )
    const current = this.currentOption(row)
    const render = (option: ResearchObjectTargetOption) => html`
      <option
        value=${`pos:${option.pos}`}
        ?selected=${current?.pos === option.pos}
      >
        ${this.optionLabel(option)}
      </option>
    `

    return html`
      ${recommended.length
        ? html`<optgroup label="Recommended">
            ${recommended.map(render)}
          </optgroup>`
        : nothing}
      ${others.length
        ? html`<optgroup label="Other research objects">
            ${others.map(render)}
          </optgroup>`
        : nothing}
    `
  }

  private renderRow(row: EditNodeRelationDraft, index: number) {
    const external = this.isExternal(row, index)
    const current = this.currentOption(row)

    return html`
      <div class="stencila-edit-relations-row">
        <select
          class="stencila-edit-node-properties-input"
          aria-label="Relation kind"
          @change=${(event: Event) => this.updateKind(index, event)}
        >
          ${this.renderKindOptions(row)}
        </select>
        <select
          class="stencila-edit-node-properties-input"
          aria-label="Relation target"
          @change=${(event: Event) => this.updateTarget(index, event)}
        >
          <option value="" ?selected=${!external && !current}>
            Select target…
          </option>
          ${this.renderTargetOptions(row)}
          <option value=${EXTERNAL_TARGET_VALUE} ?selected=${external}>
            External URI, DOI, or graph id…
          </option>
        </select>
        <button
          type="button"
          class="stencila-edit-relations-remove"
          aria-label="Remove relation"
          @click=${() => this.removeRow(index)}
        >
          <span class="i-lucide:trash-2" aria-hidden="true"></span>
        </button>
        ${external
          ? html`
              <input
                class="stencila-edit-node-properties-input stencila-edit-relations-external"
                aria-label="External relation target"
                .value=${row.target}
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                placeholder="https://doi.org/10.1234/example"
                @input=${(event: Event) =>
                  this.updateExternalTarget(index, event)}
              />
            `
          : nothing}
      </div>
    `
  }

  override render() {
    return html`
      <div class="stencila-edit-node-properties-field">
        <span>Relations</span>
        <div class="stencila-edit-relations-rows">
          ${this.rows.map((row, index) => this.renderRow(row, index))}
          <button
            type="button"
            class="stencila-edit-node-properties-action stencila-edit-relations-add"
            @click=${this.addRow}
          >
            <span class="i-lucide:plus" aria-hidden="true"></span>
            <span>Add relation</span>
          </button>
        </div>
      </div>
      ${this.error
        ? html`<div class="stencila-edit-node-properties-error" role="alert">
            ${this.error}
          </div>`
        : nothing}
    `
  }
}
