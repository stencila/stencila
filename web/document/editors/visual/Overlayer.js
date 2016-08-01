'use strict';

var Overlay = require('substance/ui/Overlay');

var TextToolset = require('./TextToolset');

/**
 * Overlay toolsets over the current slection
 * 
 * This is derived from Substance `Overlay` to implement
 * alternative positioning. Instead of using the Substance class `DefaultOverlay`
 * this class renders toolsets directly within the overlay element
 *
 * @class      Overlayer (name)
 */
function Overlayer() {
  Overlayer.super.apply(this, arguments);

  /**
   * Keep track of position because this can be lost
   * when this overlay is rerendered but without `position()`
   * being called for certain documents events.
   */
  this.top = 0;
  this.left = 0;
}

Overlayer.Prototype = function() {

  var _super = Overlayer.super.prototype;

  this.render = function($$) {
	  var el = $$('div')
      .addClass('sc-overlay')
      .css('top', this.top + 'px')
      .css('left', this.left + 'px');
    el.append(
      $$(TextToolset).ref('textToolset')
    );
    return el;
  }

  // Override of `position()` method
  this.position = function(hints) {
    console.log(hints);
    if (hints) {
      var contentWidth = this.el.htmlProp('offsetWidth');
      var selectionMaxWidth = hints.rectangle.width;

      // By default, Overlay are aligned center/bottom to the selection
      var top = hints.rectangle.top + hints.rectangle.height;
      this.el.css('top', top);
      var leftPos = hints.rectangle.left + selectionMaxWidth/2 - contentWidth/2;
      // Must not exceed left bound
      leftPos = Math.max(leftPos, 0);
      // Must not exceed right bound
      var maxLeftPos = hints.rectangle.left + selectionMaxWidth + hints.rectangle.right - contentWidth;
      leftPos = Math.min(leftPos, maxLeftPos);
      this.el.css('left', leftPos);

      this.top = top;
      this.left = leftPos;
    }
  };

};

Overlay.extend(Overlayer);

module.exports = Overlayer;
