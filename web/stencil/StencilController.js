'use strict';

var TwoPanelController = require('substance-fe0ed/ui/TwoPanelController');

var I18n = require('substance-fe0ed/ui/i18n');
I18n.instance.load(require('../i18n/en'));

function StencilController() {
  StencilController.super.apply(this, arguments);
}

StencilController.Prototype = function() {

  this.getContentPanel = function() {
    return this.refs.contentPanel;
  };

  this.renderDocument = function() {
    var doc = this.getDocument();
    var logger = this.getLogger();

    if (!doc.__isRendering) {
      logger.info('Rendering ...');

      doc.__isRendering = true;
      // Pass saving logic to the user defined callback if available
      if (this.props.onRender) {
        this.props.onRender(doc, function(err) {
          doc.__isRendering = false;
          if (err) {
            logger.error(err.message || err.toString());
          } else {
            // HACK: this is there to update the RenderTool
            // it only works because the current implementation
            // in ToolManager updates all tools when document:saved
            // is fired.
            this.emit('document:saved');
            logger.info('No changes');
          }
        }.bind(this));
      } else {
        logger.error('renderDocument is not handled at the moment. Make sure onRender is passed in the props');
      }
    }
  };

  this.handleStateUpdate = function(newState) {
    function getActiveNodes(state) {
      if (state.contextId === 'edit-source') {
        return [ state.nodeId ];
      }
      return [];
    }
    var activeAnnos = getActiveNodes(newState);
    setTimeout(function() {
      this.contentHighlights.set({
        'stencil': activeAnnos
      });
    }.bind(this));
  };

};

TwoPanelController.extend(StencilController);

module.exports = StencilController;
