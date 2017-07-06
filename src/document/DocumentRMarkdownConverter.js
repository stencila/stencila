import DocumentXMarkdownConverter from './DocumentXMarkdownConverter'

export default class DocumentRMarkdownConverter extends DocumentXMarkdownConverter {

  static match (fileName) {
    return DocumentXMarkdownConverter.match(fileName, ['rmd'])
  }

}
