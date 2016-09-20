'use strict';

import TextBlockComponent from 'substance/ui/TextBlockComponent'

function TitleComponent () {
  TitleComponent.super.apply(this, arguments);
}

TitleComponent.Prototype = function () {
  var _super = TitleComponent.super.prototype;

  this.render = function ($$) {
    return _super.render.call(this, $$)
                        .addClass('sc-title');
  };
};

TextBlockComponent.extend(TitleComponent);

module.exports = TitleComponent;
