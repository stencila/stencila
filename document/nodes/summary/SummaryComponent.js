'use strict';

import TextBlockComponent from 'substance/ui/TextBlockComponent'

function SummaryComponent () {
  SummaryComponent.super.apply(this, arguments);
}

SummaryComponent.Prototype = function () {
  var _super = SummaryComponent.super.prototype;

  this.render = function ($$) {
    return _super.render.call(this, $$)
                        .addClass('sc-summary');
  };
};

TextBlockComponent.extend(SummaryComponent);

module.exports = SummaryComponent;
