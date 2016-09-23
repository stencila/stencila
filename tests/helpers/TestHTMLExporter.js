import HTMLExporter from 'substance/model/HTMLExporter'

function TestHTMLExporter (DocumentClass, converters) {
  TestHTMLExporter.super.call(this, {
    DocumentClass: DocumentClass,
    schema: DocumentClass.schema,
    converters: converters
  })
}

TestHTMLExporter.Prototype = function () {
  this.exportDocument = function (doc) {
    var bodyNodes = this.convertContainer(doc.get('content'))
    var wrapper = this.$$('div').append(bodyNodes)
    return wrapper.html()
  }
}

HTMLExporter.extend(TestHTMLExporter)

export default TestHTMLExporter
