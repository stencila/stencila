import ComponentClient from '../component/ComponentClient'

class DocumentClient extends ComponentClient {

  save (content, format) {
    return this.call('save', {
      content: content,
      format: format || 'html'
    })
  }

}

export default DocumentClient
