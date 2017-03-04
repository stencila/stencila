import * as cheerio from 'cheerio'

import {md2html, html2md} from '../utilities/markdown'

/**
 * Markdown converter for the `Document` class
 *
 * Currently, Github Flavored Markdown is assumed for loading and dumping.
 */
class DocumentMarkdownConverter {
  /**
   * Load a document from Markdown
   *
   * @param  {Document} doc Document to load
   * @param  {[type]} content  Markdown content
   * @param  {[type]} options  Any options (see implementations for those available)
   */
  load (doc, content, options) {
    let html = md2html(content)
    doc.content = cheerio.load(html)
  }

  /**
   * Dump a document to Markdown
   *
   * Leading and trailing whitespace, including newlines, are trimmed
   *
   * @param  {Document} doc Document to dump
   * @param  {Object} options  Any options (see implementations for those available)
   * @returns {String}          Content of the document as Commonmark
   */
  dump (doc, options) {
    let html = doc.content.html()
    return html2md(html)
  }
}

export default DocumentMarkdownConverter
