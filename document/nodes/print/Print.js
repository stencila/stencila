import InlineNode from 'substance/model/InlineNode'

class Print extends InlineNode {

  refresh () {
    if (this.source) {
      // TODO
      // Evaluate the source within the document's current
      // context
      try {
        this.content = this.document.write(this.source)
        this.error = false
      } catch (error) {
        this.content = error.toString()
        this.error = true
      }
      this.emit('content:changed')
    }
  }

}

Print.define({
  type: 'print',
  source: { type: 'string', optional: true },
  error: { type: 'boolean', default: false },
  content: { type: 'string', optional: true }
})

export default Print
