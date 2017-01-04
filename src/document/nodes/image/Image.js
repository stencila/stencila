import DocumentNode from 'substance/model/DocumentNode'

class Image extends DocumentNode {}

Image.define({
  type: 'image',
  src: { type: 'string' }
})

export default Image
