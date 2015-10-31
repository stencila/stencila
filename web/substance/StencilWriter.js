'use strict';

var oo = require('substance/util/oo');
var StencilController = require('./StencilController');
var ContextToggles = require('substance/ui/ContextToggles');
var ContentPanel = require("substance/ui/ContentPanel");
var StatusBar = require("substance/ui/StatusBar");
var Toolbar = require('substance/ui/Toolbar');
var WriterTools = require('./packages/writer/WriterTools');
var ContainerEditor = require('substance/ui/ContainerEditor');
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
    commands: [
      require('substance/ui/SelectAllCommand'),

      // Special commands
      require('substance/packages/embed/EmbedCommand'),

      require('substance/packages/text/SwitchTextTypeCommand'),
      require('substance/packages/strong/StrongCommand'),
      require('substance/packages/emphasis/EmphasisCommand'),
      require('substance/packages/link/LinkCommand')
    ],
    textTypes: [
      {name: 'paragraph', data: {type: 'paragraph'}},
      {name: 'heading1',  data: {type: 'heading', level: 1}},
      {name: 'heading2',  data: {type: 'heading', level: 2}},
      {name: 'heading3',  data: {type: 'heading', level: 3}}
    ]
  },
  panelOrder: ['toc'],
  containerId: 'body',
  isEditable: true
};

function StencilWriter(parent, params) {
  StencilController.call(this, parent, params);
}

StencilWriter.Prototype = function() {

  this.static = {
    config: CONFIG
  };

  this.render = function() {
    var doc = this.props.doc;
    var config = this.getConfig();
    var el = $$('div').addClass('lc-writer sc-controller');

    el.append(
      $$('div').ref('workspace').addClass('le-workspace').append(
        // Main (left column)
        $$('div').ref('main').addClass("le-main").append(
          $$(Toolbar).ref('toolbar').append($$(WriterTools)),

          $$(ContentPanel).append(
            // The full fledged document (ContainerEditor)
            $$("div").ref('body').addClass('document-content').append(
              $$(ContainerEditor, {
                name: 'body',
                containerId: config.containerId,
                editable: false,
                commands: config.body.commands,
                textTypes: config.body.textTypes
              }).ref('bodyEditor')
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

oo.inherit(StencilWriter, StencilController);

module.exports = StencilWriter;
