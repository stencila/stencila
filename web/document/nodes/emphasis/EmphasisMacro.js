'use strict';

var AnnotationMacro = require('../../ui/AnnotationMacro');

/**
 * A macro for creating `Emphasis` nodes
 * 
 * Uses enclosing underscores.
 * 
 * Note that this is different to Markdown which uses single asterisk or single underscores.
 * 
 * @class      EmphasisMacro (name)
 */
function EmphasisMacro () {
};

EmphasisMacro.Prototype = function() {

  this.appliesTo = [];

  this.regex =  /\_([^\_]+)\_/;
  
  this.createNodeData = function(match) {
    return {
      type: 'emphasis',
      text: match[1]
    };
  };

};

AnnotationMacro.extend(EmphasisMacro);

module.exports = EmphasisMacro;
