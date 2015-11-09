'use strict';

var _ = require('substance/util/helpers');

var oo = require('substance/util/oo');
var Controller = require("substance/ui/Controller");
var Component = require('substance/ui/Component');
var $$ = Component.$$;

// Substance is i18n ready, but by now we did not need it
// Thus, we configure I18n statically as opposed to loading
// language files for the current locale
var I18n = require('substance/ui/i18n');
I18n.instance.load(require('substance/i18n/en'));
I18n.instance.load(require('./i18n/en'));
// e.g. in german
// I18n.instance.load(require('substance/ui/i18n/de'));
// I18n.instance.load(require('./i18n/de'));

function LensController(parent, params) {
  Controller.call(this, parent, params);

  this.handleApplicationKeyCombos = this.handleApplicationKeyCombos.bind(this);

  // action handlers
  this.actions({
    "switchState": this.switchState,
    "switchContext": this.switchContext,
    "toggleBibItem": this.toggleBibItem
  });
}

LensController.Prototype = function() {

  // Action used by BibItemComponent when clicked on focus
  this.toggleBibItem = function(bibItem) {
    if (this.state.bibItemId === bibItem.id) {
      this.setState({
        contextId: 'bib-items'
      });
    } else {
      this.setState({
        contextId: 'bib-items',
        bibItemId: bibItem.id
      });
    }
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

  // Some things should go into controller
  this.getChildContext = function() {
    var childContext = Controller.prototype.getChildContext.call(this);

    return _.extend(childContext, {
      i18n: I18n.instance
    });
  };

  this.getInitialState = function() {
    return {'contextId': 'toc'};
  };

  // Action handlers
  // ---------------

  // handles 'switch-state'
  this.switchState = function(newState, options) {
    options = options || {};
    this.setState(newState);
    if (options.restoreSelection) {
      this.restoreSelection();
    }
  };

  // handles 'switch-context'
  this.switchContext = function(contextId, options) {
    options = options || {};
    this.setState({ contextId: contextId });
    if (options.restoreSelection) {
      this.restoreSelection();
    }
  };

  this.restoreSelection = function() {
    var surface = this.getSurface('body');
    surface.rerenderDomSelection();
  };

  // Pass writer start
  this._panelPropsFromState = function (state) {
    var props = _.omit(state, 'contextId');
    props.doc = this.props.doc;
    return props;
  };

  this.getActivePanelElement = function() {
    var ComponentClass = this.componentRegistry.get(this.state.contextId);
    if (ComponentClass) {
      return $$(ComponentClass, this._panelPropsFromState(this.state)).ref('contextPanel');
    } else {
      console.warn("Could not find component for contextId:", this.state.contextId);
    }
  };

  this.uploadFile = function(file, cb) {
    // This is a testing implementation
    if (this.props.onUploadFile) {
      return this.props.onUploadFile(file, cb);
    } else {
      // Default file upload implementation
      // We just return a temporary objectUrl
      var fileUrl = window.URL.createObjectURL(file);
      cb(null, fileUrl);
    }
  };

  this.renderContextPanel = function() {
    var panelElement = this.getActivePanelElement();
    if (!panelElement) {
      return $$('div').append("No panels are registered");
    } else {
      return $$('div').append(panelElement);
    }
  };


  // Hande Writer state change updates
  // --------------
  //
  // Here we update highlights

  this.handleStateUpdate = function(newState) {
    // var oldState = this.state;
    var doc = this.getDocument();

    function getActiveNodes(state) {
      if (state.contextId === 'editSource') { 
        return [ state.nodeId ];
      }
      return [];
    }

    var activeAnnos = getActiveNodes(newState);
    // HACK: updates the highlights when state
    // transition has finished
    setTimeout(function() {
      doc.setHighlights(activeAnnos);  
    }, 0);
  };

};

oo.inherit(LensController, Controller);

module.exports = LensController;