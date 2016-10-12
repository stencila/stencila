import AnnotationMacro from '../../ui/AnnotationMacro'

/**
 * A macro for creating `Emphasis` nodes
 *
 * Uses enclosing underscores.
 *
 * Note that this is different to Markdown which uses single asterisk or single underscores.
 *
 * @class      EmphasisMacro (name)
 */
class EmphasisMacro extends AnnotationMacro {

  get regex () {
    return /_([^_]+)_/
  }

  createNodeData (match) {
    return {
      type: 'emphasis',
      text: match[1]
    }
  }

}

export default EmphasisMacro
