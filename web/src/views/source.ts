import { 
  LanguageDescription,
  LanguageSupport,
  syntaxHighlighting,
  defaultHighlightStyle,
  StreamLanguage
} from '@codemirror/language';
import { Extension } from '@codemirror/state';
import { 
  EditorView as CodeMirrorView,
  lineNumbers,
  highlightActiveLineGutter
} from "@codemirror/view";
import { LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CodeMirrorClient } from "../clients/codemirror";
import { type DocumentAccess } from "../types";

import "./source.css";


/**
 * Source code editor for a document
 *
 * A view which provides read-write access to the document using
 * a particular format.
 */
@customElement("stencila-source-view")
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
   * A read-write client which sends and receives string patches
   * for the source code to and from the server
   */
  private codeMirrorClient: CodeMirrorClient;

  /**
   * A CodeMirror editor view which the client interacts with
   */
  private codeMirrorView: CodeMirrorView;

  /**
   * default language description
   * set as 'markdown'
   */
  private defaultLangDescription = LanguageDescription.of({
      name: 'markdown',
      extensions: ['md'],
      load: async () => {
        return import('@codemirror/lang-markdown').then((obj) => obj.markdown())
      }
  })
  
  /**
   * Array of `LanguageDescription` objects available for the edit view
   */
  static languageDescriptions  = [
    LanguageDescription.of({
      name: 'jats',
      extensions: ['jats.xml'],
      load: async () => {
        return import('@codemirror/lang-xml').then((obj) => obj.xml())
      }
    }),
    LanguageDescription.of({
      name: 'json',
      extensions: ['json'],
      load: async () => {
        return import('@codemirror/lang-json').then((obj) => obj.json())
      }
    }),
    LanguageDescription.of({
      name: 'json5',
      extensions: ['json5'],
      load: async () => {
        return import('codemirror-json5').then(obj => obj.json5())
      }
    }),
    LanguageDescription.of({
      name: 'html',
      extensions: ['html'],
      load: async () => {
        return import('@codemirror/lang-html').then((obj) => obj.html())
      }
    }),
    LanguageDescription.of({
      name: 'yaml',
      extensions: ['yaml', 'yml'],
      load: async () => {
        return import('@codemirror/legacy-modes/mode/yaml').then((yml) => 
          new LanguageSupport(StreamLanguage.define(yml.yaml))
        )
      }
    }),
  ]

  /**
   * Get the required `LanguageSupport` for the format of the source view
   * 
   * default to `defaultLangDescription` if format does not exist
   * @param {string} format `format` parameter of the source view
   * @returns `LanguageSupport` instance
   */
  private async getLanguageExtension(format: string): Promise<LanguageSupport> {
    const ext = LanguageDescription.matchLanguageName(
      SourceView.languageDescriptions,
      format
    ) ?? this.defaultLangDescription
  
    return await ext.load()
  }

  private async getViewExtensions(): Promise<Extension[]> {
    const langExt = await this.getLanguageExtension(this.format)

    return [
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      langExt,
      lineNumbers(),
      highlightActiveLineGutter()
    ]
  }

  /**
   * Override so that the `CodeMirrorView` is instantiated _after_ this
   * element has a `renderRoot`.
   */
  override connectedCallback() {
    super.connectedCallback();
    
    this.getViewExtensions().then((extensions) =>{
      this.codeMirrorClient = new CodeMirrorClient(this.id, this.access, this.format);
      this.codeMirrorView = new CodeMirrorView({
        extensions: [
          this.codeMirrorClient.sendPatches(), 
          ...extensions
        ],
        parent: this.renderRoot,
      })
      this.codeMirrorClient.receivePatches(this.codeMirrorView);
    })
  }
}
