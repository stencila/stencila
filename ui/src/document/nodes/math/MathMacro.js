import InlineNodeMacro from '../../ui/InlineNodeMacro'

class MathMacro extends InlineNodeMacro {

  get regex () {
    // Allow for both AsciiMath pipe delimeters (|) and
    // TeX dollar ($) delimiters. In both cases the start and end delimiters
    // must be followed/preceded by a non-space character. For TeX, the first
    // dollar must not be followed by a digit.
    //                2                   5
    return /(\|(\S|(\S.*\S))\|)|(\$(([^0-9\s])|([^0-9\s].*\S))\$)/
  }

  createNodeData (match) {
    var source, language, display
    if (match[2]) {
      source = match[2]
      language = 'asciimath'
    } else if (match[5]) {
      source = match[5]
      language = 'tex'
    } else {
      throw new Error('No match!')
    }

    return {
      type: 'math',
      source: source,
      language: language,
      display: display
    }
  }

}

export default MathMacro
