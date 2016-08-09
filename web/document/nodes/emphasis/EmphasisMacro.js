'use strict';

var AnnotationMacro = require('../../ui/AnnotationMacro');

/**
 * A macro for creating `Emphasis` nodes
 * 
 * Uses enclosing underscores.
 * 
 * Note that this is different to Markdown which uses single asterisk or single underscores.
 * 
 * @class      CodeMacro (name)
 */
function CodeMacro () {
};

CodeMacro.Prototype = function() {

  this.appliesTo = [];

  this.regex =  /\_([^\_]+)\_/;
  
  this.createNodeData = function(match) {
  	console.log(match[0]);
    return {
      type: 'emphasis',
      text: match[1]
    };
  };

};

AnnotationMacro.extend(CodeMacro);

module.exports = CodeMacro;
