'use strict';

var InlineNodeMacro = require('../../ui/InlineNodeMacro');

function MathMacro () {
};

MathMacro.Prototype = function () {
  // Allow for both AsciiMath pipe delimeters (|) and
  // TeX dollar ($) delimiters. In both cases the start and end delimiters
  // must be followed/preceded by a non-space character. For TeX, the first
  // dollar must not be followed by a digit.
  //                2                   5
  this.regex = /(\|(\S|(\S.*\S))\|)|(\$(([^0-9\s])|([^0-9\s].*\S))\$)/;

  this.createNodeData = function (match) {
    var source, language, display;
    if (match[2]) {
      source = match[2];
      language = 'asciimath';
    } else if (match[5]) {
      source = match[5];
      language = 'tex';
    } else {
      throw new Error('No match!');
    }

    return {
      type: 'math',
      source: source,
      language: language,
      display: display
    };
  };
};

InlineNodeMacro.extend(MathMacro);

module.exports = MathMacro;
