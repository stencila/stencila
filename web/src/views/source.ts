import {
  autocompletion,
  startCompletion,
  completionKeymap,
} from "@codemirror/autocomplete";
import { history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  foldGutter,
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  LanguageDescription,
  LanguageSupport,
  syntaxHighlighting,
  StreamLanguage,
} from "@codemirror/language";
import { searchKeymap, search } from "@codemirror/search";
import { Extension, Compartment, StateEffect } from "@codemirror/state";
import {
  dropCursor,
  EditorView as CodeMirrorView,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { html, css, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CodeMirrorClient } from "../clients/codemirror";
import { withTwind } from "../twind";
import type { DocumentId, DocumentAccess } from "../types";

const FORMATS = {
  markdown: "Markdown",
  html: "HTML",
  jats: "JATS",
  json: "JSON",
  jsonld: "JSON-LD",
  json5: "JSON5",
  yaml: "YAML",
};

/**
 * Source code editor for a document
 *
 * A view which provides read-write access to the document using
 * a particular format.
 */
@customElement("stencila-source-view")
@withTwind()
export class SourceView extends LitElement {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId;

  /**
   * The access level of the editor
   *
   * This property is passed through to the `CodeMirrorClient`
   * and used to determine whether or not the document is
   * read-only or writable.
   *
   * This should not be `edit`, `write` or `admin` since this view
   * does not provide the means to modify those.
   */
  @property()
  access: DocumentAccess = "write";

  /**
   * The format of the source code
   */
  @property()
  format: string = "markdown";

  /**
   * Turn on/off editor line wrapping
   */
  @property({ attribute: "line-wrapping", type: Boolean })
  lineWrap: boolean = true;

  /**
   * A read-write client which sends and receives string patches
   * for the source code to and from the server
   */
  private codeMirrorClient: CodeMirrorClient;

  /**
   * A CodeMirror editor view which the client interacts with
   */
  private codeMirrorView: CodeMirrorView;

  /**
   * `Compartment` for setting `CodeMirrorView.lineWrapping` extension
   */
  private lineWrappingConfig = new Compartment();

  /**
   * Array of CodeMirror `LanguageDescription` objects available for the edit view
   *
   * Note: The first language description is used as the default.
   */
  static languageDescriptions = [
    LanguageDescription.of({
      name: "markdown",
      extensions: ["md"],
      load: async () => {
        return import("@codemirror/lang-markdown").then((obj) =>
          obj.markdown(),
        );
      },
    }),
    LanguageDescription.of({
      name: "jats",
      extensions: ["jats.xml"],
      load: async () => {
        return import("@codemirror/lang-xml").then((obj) => obj.xml());
      },
    }),
    LanguageDescription.of({
      name: "json",
      extensions: ["json"],
      load: async () => {
        return import("@codemirror/lang-json").then((obj) => obj.json());
      },
    }),
    LanguageDescription.of({
      name: "json5",
      extensions: ["json5"],
      load: async () => {
        return import("codemirror-json5").then((obj) => obj.json5());
      },
    }),
    LanguageDescription.of({
      name: "html",
      extensions: ["html"],
      load: async () => {
        return import("@codemirror/lang-html").then((obj) => obj.html());
      },
    }),
    LanguageDescription.of({
      name: "yaml",
      extensions: ["yaml", "yml"],
      load: async () => {
        return import("@codemirror/legacy-modes/mode/yaml").then(
          (yml) => new LanguageSupport(StreamLanguage.define(yml.yaml)),
        );
      },
    }),
  ];

  /**
   * Dispatch a CodeMirror `StateEffect` to the editor
   */
  private dispatchEffect(effect: StateEffect<unknown>) {
    const docState = this.codeMirrorView?.state;

    const transaction =
      docState?.update({
        effects: [effect],
      }) ?? {};

    this.codeMirrorView?.dispatch(transaction);
  }

  /**
   * Get the CodeMirror `LanguageSupport` for a particular format
   *
   * Defaults to the first `SourceView.languageDescriptions` if it does no
   * matching language extension is found.
   *
   * @param {string} format `format` parameter of the source view
   * @returns `LanguageSupport` instance
   */
  private async getLanguageExtension(format: string): Promise<LanguageSupport> {
    const ext =
      LanguageDescription.matchLanguageName(
        SourceView.languageDescriptions,
        format,
      ) ?? SourceView.languageDescriptions[0];

    return await ext.load();
  }

  /**
   * Get the CodeMirror editor view extensions
   */
  private async getViewExtensions(): Promise<Extension[]> {
    const langExt = await this.getLanguageExtension(this.format);

    const lineWrapping = this.lineWrappingConfig.of(
      CodeMirrorView.lineWrapping,
    );

    const keyMaps = keymap.of([
      indentWithTab,
      ...historyKeymap,
      ...completionKeymap,
      ...searchKeymap,
      { key: "Ctrl-Space", run: startCompletion },
    ]);

    return [
      langExt,
      keyMaps,
      history(),
      search({ top: true }),
      lineNumbers(),
      foldGutter(),
      lineWrapping,
      dropCursor(),
      highlightActiveLineGutter(),
      indentOnInput(),
      highlightSpecialChars(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      autocompletion(),
    ];
  }

  /**
   * Override `LitElement.update` to dispatch any changes to editor config
   * to the editor.
   */
  override async update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties);

    if (changedProperties.has("format")) {
      // Destroy the existing editor if there is one
      this.codeMirrorView?.destroy();

      // Setup client and editor for the format
      this.getViewExtensions().then((extensions) => {
        this.codeMirrorClient = new CodeMirrorClient(
          this.doc,
          this.access,
          this.format,
        );

        this.codeMirrorView = new CodeMirrorView({
          extensions: [this.codeMirrorClient.sendPatches(), ...extensions],
          parent: this.renderRoot.querySelector("#codemirror"),
        });

        this.codeMirrorClient.receivePatches(this.codeMirrorView);
      });
    }

    if (changedProperties.has("lineWrap")) {
      this.dispatchEffect(
        this.lineWrappingConfig.reconfigure(
          this.lineWrap ? CodeMirrorView.lineWrapping : [],
        ),
      );
    }
  }

  /**
   * CSS styling for the CodeMirror editor
   *
   * Overrides some of the default styles used by CodeMirror.
   */
  static styles = css`
    .cm-editor {
      border: 1px solid rgb(189, 186, 186);
      height: 100vh;
    }
  `;

  render() {
    return html`
      <div>
        <div>${this.renderControls()}</div>
        <div id="codemirror"></div>
      </div>
    `;
  }

  private renderControls() {
    return html`
      <div class="mb-4 flex">
        <div>${this.renderFormatSelect()} ${this.renderLineWrapCheckbox()}</div>
      </div>
    `;
  }

  private renderFormatSelect() {
    return html`
      <label>
        Format
        <select
          @change=${(e: Event) =>
            (this.format = (e.target as HTMLSelectElement).value)}
        >
          ${Object.entries(FORMATS).map(
            ([format, name]) =>
              html`<option value=${format} ?selected=${this.format === format}>
                ${name}
              </option>`,
          )}
        </select>
      </label>
    `;
  }

  private renderLineWrapCheckbox() {
    return html`
      <label class="text-sm">
        ${"Enable line wrapping"}
        <input
          type="checkbox"
          class="ml-1"
          ?checked="${this.lineWrap}"
          @change="${(e: Event) =>
            (this.lineWrap = (e.target as HTMLInputElement).checked)}"
        />
      </label>
    `;
  }
}
