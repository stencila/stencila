'use strict';

var InlineNodeMacro = require('../../ui/InlineNodeMacro');


function EmojiMacro () {
};

EmojiMacro.Prototype = function() {

  // First semicolon must not be followed by space,
  // last semicolon must not be preceeded by space                   
  this.regex =  /\:(\S|(\S.*\S))\:/;
  
  this.createNodeData = function(match) {
    var name = match[1];    
    return {
      type: 'emoji',
      name: name
    };
  };

};

InlineNodeMacro.extend(EmojiMacro);

module.exports = EmojiMacro;
