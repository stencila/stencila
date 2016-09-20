'use strict';

import Overlay from 'substance/ui/OverlayContainer'
import getRelativeBoundingRect from 'substance/util/getRelativeBoundingRect'

var TextToolset = require('./TextToolset');
var BlockToolset = require('./BlockToolset');

/**
 * Overlay toolsets over the current slection
 *
 * This is derived from Substance `Overlay` to implement
 * alternative positioning. Instead of using the Substance class `DefaultOverlay`
 * this class renders toolsets directly within the overlay element
 *
 * @class      Overlayer (name)
 */
function Overlayer () {
  Overlayer.super.apply(this, arguments);

  /**
   * Keep track of position because this can be lost
   * when this overlay is rerendered but without `position()`
   * being called for certain documents events.
   */
  this.top = 0;
  this.left = 0;
}

Overlayer.Prototype = function () {
  this.render = function ($$) {
    var el = $$('div')
      .addClass('sc-overlay')
      .css('top', this.top + 'px')
      .css('left', this.left + 'px');
    el.append(
      $$(TextToolset).ref('textToolset'),
      $$(BlockToolset).ref('blockToolset')
    );
    return el;
  };

  // Override of `position()` method to determin a position for this overlay
  this.position = function (hints) {
    // Get selection rectangle so that overlay can be positioned relative to it
    // At time of writing `Surface.getBoundingRectangleForSelection` was experimental and
    // sometimes created a hints.rectangle that was an empty object (e.g. for node selections). This checks for that
    // and if necessary tries to work out it's own selection rectangle
    var selected = null;
    if (hints) {
      if (hints.rectangle) {
        if (hints.rectangle.top) {
          selected = hints.rectangle;
        }
      }
    }
    if (!selected) {
      var surface = this.context.surfaceManager.getFocusedSurface();
      var selection = surface.getSelection();
      if (selection) {
        if (selection.isNodeSelection()) {
          var nodeId = selection.getNodeId();
          console.warn('No selection rectangle provided for {' + nodeId + '}, attempting to get one');
          var componentEl = document.querySelector('[data-id=' + nodeId + ']');
          var containerEl = this.context.scrollPane.refs.content.el.el;
          if (componentEl && containerEl) {
            selected = getRelativeBoundingRect(componentEl, containerEl);
          }
        }
      }
    }

    if (selected) {
      var overlay = {
        height: this.el.htmlProp('offsetHeight'),
        width: this.el.htmlProp('offsetWidth')
      };

      // By default, aligned top/center to the selected
      var top = selected.top - overlay.height - 3;
      var left = selected.left + selected.width / 2 - overlay.width / 2;
      // Must not exceed left bound
      left = Math.max(left, 0);
      // Must not exceed right bound
      var maxLeftPos = selected.left + selected.width + selected.right - overlay.width;
      left = Math.min(left, maxLeftPos);

      // Change position
      this.el.css('top', top);
      this.el.css('left', left);

      // Store position for next rendering
      this.top = top;
      this.left = left;
    } else {
      console.warn('No selection rectangle to position overlay');
    }
  };
};

Overlay.extend(Overlayer);

module.exports = Overlayer;
