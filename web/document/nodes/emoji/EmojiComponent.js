'use strict';

var Component = require('substance/ui/Component');

var emojione = require('emojione');
// Consistent with making everying served locally (for offline use etc)...
emojione.imagePathPNG = '/get/web/emojione/png/';

function EmojiComponent () {

  EmojiComponent.super.apply(this, arguments);

}

EmojiComponent.Prototype = function () {

  this.didMount = function () {

    this.props.node.on('name:changed', this.rerender, this);

  };

  this.dispose = function () {

    this.props.node.off(this);

  };

  this.render = function ($$) {

    var node = this.props.node;
    var el = $$('span')
      .addClass('sc-emoji');
    var shortname = ':' + node.name + ':';
    var img = emojione.shortnameToImage(shortname);
    if (img === shortname) {

      // Emoji name is not matched. Indicate this
      // but show name to reflect user intent
      el.addClass('sm-unknown')
        .text(shortname);

    } else {

      // Emoji found so append `img` tag produced by EmojiOne
      el.html(img);

    }
    return el;

  };

};

Component.extend(EmojiComponent);

module.exports = EmojiComponent;
