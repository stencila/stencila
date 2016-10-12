import BlockNodeMacro from '../../ui/BlockNodeMacro'

/**
 * A macro for creating `Image` nodes.
 *
 * Uses Markdown syntax:
 *
 *   ![Alternative text](/path/to/img.jpg "Title")
 *
 * simplified to:
 *
 *   ![](/path/to/img.jpg)
 *
 * because `Image` nodes currently don't have `alt` and `title`
 * properties.
 *
 * Only applies at start of paragraph (i.e. only block images)
 *
 * @class      ImageMacro (name)
 */
class ImageMacro extends BlockNodeMacro {

  get regex () {
    return /^!\[\]\(([^\)]*)\)$/
  }

  createNodeData (match) {
    return {
      type: 'image',
      src: match[1]
    }
  }

}

export default ImageMacro
