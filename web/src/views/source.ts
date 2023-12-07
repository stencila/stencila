import {
  history,
  historyKeymap,
  indentWithTab,
} from '@codemirror/commands'
import {
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  LanguageDescription,
  LanguageSupport,
  syntaxHighlighting,
  StreamLanguage,
} from "@codemirror/language";
import { 
  Extension,
  Compartment,
  StateEffect,
  // EditorState
} from "@codemirror/state";
import {
  EditorView as CodeMirrorView,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CodeMirrorClient } from "../clients/codemirror";
import { installTwind } from "../twind";
import { type DocumentAccess } from "../types";


const withTwind = installTwind()

/**
 * Source code editor for a document
 *
 * A view which provides read-write access to the document using
 * a particular format.
 */
@customElement("stencila-source-view")
@withTwind
export class SourceView extends LitElement {
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
   * Allow for line wrapping
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
  private lineWrappingConfig = new Compartment()

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
          obj.markdown()
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
          (yml) => new LanguageSupport(StreamLanguage.define(yml.yaml))
        );
      },
    }),
  ];

  /**
   * Dispatch a CodeMirror `StateEffect` to the editor
   */
  private dispatchEffect(effect: StateEffect<unknown>) {
    const docState = this.codeMirrorView?.state

    const transaction =
      docState?.update({
        effects: [effect],
      }) ?? {}

    this.codeMirrorView?.dispatch(transaction)
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
        format
      ) ?? SourceView.languageDescriptions[0];

    return await ext.load();
  }

  protected async update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties)

    if (changedProperties.has('lineWrap')) {
      this.dispatchEffect(this.lineWrappingConfig.reconfigure(
        this.lineWrap
          ? CodeMirrorView.lineWrapping 
          : []
      ))
    }
  }

  /**
   * Get the CodeMirror editor view extensions
   */
  private async getViewExtensions(): Promise<Extension[]> {
    const langExt = await this.getLanguageExtension(this.format);

    const lineWrapping = this.lineWrappingConfig.of(CodeMirrorView.lineWrapping)

    return [
      indentOnInput(),
      history(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      lineNumbers(),
      highlightActiveLineGutter(),
      keymap.of([indentWithTab, ...historyKeymap]), 
      langExt,
      lineWrapping,
    ];
  }

  /**
   * Override so that the `CodeMirrorView` is instantiated _after_ this
   * element has a `renderRoot`.
   */
  override connectedCallback() {
    super.connectedCallback();

    this.getViewExtensions().then((extensions) => {
      this.codeMirrorClient = new CodeMirrorClient(
        this.id,
        this.access,
        this.format
      );
      this.codeMirrorView = new CodeMirrorView({
        extensions: [this.codeMirrorClient.sendPatches(), ...extensions],
        parent: this.renderRoot.querySelector("#codemirror"),
      });
      this.codeMirrorClient.receivePatches(this.codeMirrorView);
    });
  }

  /**
   * CSS styling for the CodeMirror editor
   * 
   * Overrides some of the default styles used by CodeMirror.
   */
  static css = css`
    .cm-editor {
      border: 1px solid rgb(189, 186, 186);
      height: 30vh;
    }
  `;

  private renderControls() {
    return html`
      <div class="mt-4 flex">
       <div>${this.renderLineWrapCheckbox()}</div>
      </div>
    `
  }

  private renderLineWrapCheckbox() {
    return html`
      <label class="text-sm">
        ${'Enable line wrapping'}
        <input 
          type="checkbox"
          class="ml-1"
          ?checked="${this.lineWrap}" 
          @change="${
            (e: Event) => this.lineWrap = (e.target as HTMLInputElement).checked
          }" 
        />
      </label>
    `
  }

  render() {
    return html`
      <div>
        <div id="codemirror"></div>
        <div>
          ${this.renderControls()}
        </div>
      </div>
    `;
  }
}
