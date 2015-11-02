'use strict';

var oo = require('substance/util/oo');
var StencilController = require('./StencilController');
var ContextToggles = require('substance/ui/ContextToggles');
var ContentPanel = require("substance/ui/ContentPanel");
var StatusBar = require("substance/ui/StatusBar");
var ContainerAnnotator = require('substance/ui/ContainerAnnotator');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand'),
    ],
    components: {
      "paragraph": require('substance/packages/paragraph/ParagraphComponent'),
      "heading": require('substance/packages/heading/HeadingComponent'),
      // Panels
      "toc": require('substance/ui/TocPanel')
    },
  },
  body: {
    commands: [],
  },
  panelOrder: ['toc'],
  containerId: 'body',
  isEditable: false
};

function StencilReader(parent, params) {
  StencilController.call(this, parent, params);

  this.connect(this, {
    'citation:selected': this.onCitationSelected
  });

}

StencilReader.Prototype = function() {

  this.static = {
    config: CONFIG
  };

  // this.getInitialState = function() {
  //   return {
  //     contextId: 'bib-items',
  //     citationId: 'bib_item_citation_a6da926ec7f4f4df975164f9e9ce413b',
  //   };
  // };

  this.onCitationSelected = function(citation) {
    if (this.state.citationId === citation.id) {
      this.setState({
        contextId: 'toc'
      });
      return;
    }

    if (citation.type === 'bib-item-citation') {
      this.setState({
        contextId: 'bib-items',
        citationId: citation.id
      });
    } else {
      this.setState({
        contextId: 'toc',
        citationId: citation.id
      });
    }
  };

  this.dispose = function() {
    StencilController.prototype.dispose.call(this);
    this.disconnect(this);
  };

  this.render = function() {
    var doc = this.props.doc;
    var config = this.getConfig();
    var el = $$('div').addClass('lc-reader sc-controller');

    el.append(
      $$('div').ref('workspace').addClass('le-workspace').append(
        // Main (left column)
        $$('div').ref('main').addClass("le-main").append(
          $$(ContentPanel).append(
            // The main container
            $$("div").ref('main').addClass('document-content').append(
              $$(ContainerAnnotator, {
                name: 'body',
                containerId: 'body',
                editable: false,
                commands: config.body.commands
              }).ref('bodyAnnotator')
            )
          ).ref('content')
        ),
        // Resource (right column)
        $$('div').ref('resource')
          .addClass("le-resource")
          .append(
            $$(ContextToggles, {
              panelOrder: config.panelOrder,
              contextId: this.state.contextId
            }).ref("context-toggles"),
            this.renderContextPanel()
          )
      )
    );

    // Status bar
    el.append(
      $$(StatusBar, {doc: doc}).ref('statusBar')
    );
    return el;
  };
};

oo.inherit(StencilReader, StencilController);

module.exports = StencilReader;
