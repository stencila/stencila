import { DocumentNode } from 'substance'

class Image extends DocumentNode {}

Image.define({
  type: 'image',
  src: { type: 'string' }
})

export default Image
