import ComponentDelegate from '../component/ComponentDelegate'

class DocumentDelegate extends ComponentDelegate {

  save (content, format) {
    return this.call('save', {
      content: content,
      format: format || 'html'
    })
  }

}

export default DocumentDelegate
