import DocumentMarkdownConverter from './DocumentMarkdownConverter'

/**
 * DocumentRMarkdownConverter
 *
 * A preliminary implementation of Document converter for RMarkdown
 */
export default class DocumentRMarkdownConverter extends DocumentMarkdownConverter {

  static match (fileName) {
    return fileName.slice(-4).toLowerCase() === '.rmd'
  }

  exportContent () {
    throw new Error('Not yet implemented')
  }

}
