'use strict';

var BlockNodeMacro = require('../../ui/BlockNodeMacro');

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
function ImageMacro () {
};

ImageMacro.Prototype = function() {

  this.regex =  /^\!\[\]\(([^\)]*)\)$/;
  
  this.createNodeData = function(match) {
    return {
      type: 'image',
      src: match[1]
    };
  };

};

BlockNodeMacro.extend(ImageMacro);

module.exports = ImageMacro;
