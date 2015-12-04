'use strict';

var oo = require('substance/util/oo');
var StencilController = require('./StencilController');
var ContentPanel = require("substance/ui/ContentPanel");
var StatusBar = require("substance/ui/StatusBar");
var Toolbar = require('substance/ui/Toolbar');
var WriterTools = require('./packages/writer/WriterTools');
var ContainerEditor = require('substance/ui/ContainerEditor');
var ContextSection = require('substance/ui/ContextSection');
var Component = require('substance/ui/Component');
var docHelpers = require('substance/model/documentHelpers');
var $$ = Component.$$;

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand'),
      require('./packages/writer/RenderCommand'),
    ],
    components: {
      'paragraph': require('substance/packages/paragraph/ParagraphComponent'),
      'heading': require('substance/packages/heading/HeadingComponent'),
      'link': require('./packages/link/LinkComponent'),

      'stencil-title': require('./packages/title/StencilTitleComponent'),
      'stencil-summary': require('./packages/summary/StencilSummaryComponent'),

      'stencil-math': require('./packages/math/StencilMathComponent'),
      'stencil-equation': require('./packages/equation/StencilEquationComponent'),

      'stencil-exec': require('./packages/exec/StencilExecComponent'),
      'stencil-figure': require('./packages/figure/StencilFigureComponent'),
      'stencil-text': require('./packages/text/StencilTextComponent'),

      'stencil-default-node': require('./packages/default/StencilDefaultNodeComponent'),

      // Panels
      'toc': require('substance/ui/TocPanel'),
      'cila': require('./packages/writer/CilaPanel'),
      'edit-source': require('./packages/writer/EditSourcePanel')
    }
  },
  body: {
    commands: [
      // Special commands
      require('substance/packages/embed/EmbedCommand'),
      require('substance/packages/text/SwitchTextTypeCommand'),
      require('substance/packages/strong/StrongCommand'),
      require('substance/packages/emphasis/EmphasisCommand'),
      require('substance/packages/link/LinkCommand'),
      require('./packages/table/InsertTableCommand'),
    ],
    textTypes: [
      {name: 'paragraph', data: {type: 'paragraph'}},
      {name: 'title', data: {type: 'stencil-title'}},
      {name: 'summary', data: {type: 'stencil-summary'}},
      {name: 'heading1',  data: {type: 'heading', level: 1}},
      {name: 'heading2',  data: {type: 'heading', level: 2}},
      {name: 'heading3',  data: {type: 'heading', level: 3}}
    ]
  },
  panels: {
    'toc': {
      hideContextToggles: false
    },
    'cila': {
      hideContextToggles: false
    },
    'edit-source': {
      hideContextToggles: true
    }
  },
  panelOrder: ['toc','cila'],
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

  this.onSelectionChanged = function(sel, surface) {
    var config = this.getConfig();

    function getActiveAnno(type) {
      return docHelpers.getAnnotationsForSelection(doc, sel, type, config.containerId)[0];
    }

    if (surface.name !== config.containerId) return;
    if (sel.isNull() || !sel.isPropertySelection()) {
      return;
    }
    if (sel.equals(this.prevSelection)) {
      return;
    }
    this.prevSelection = sel;
    var doc = surface.getDocument();

    // Annotation
    var stencilText = getActiveAnno('stencil-text');
    if (stencilText && stencilText.getSelection().equals(sel)) {
      // Trigger state change
      this.setState({
        contextId: 'edit-source',
        nodeId: stencilText.id
      });
    } else {
      if (this.state.contextId !== 'toc') {
        this.setState({
          contextId: 'toc'
        });
      }
    }
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
        $$(ContextSection, {
          panelProps: this._panelPropsFromState(),
          contextId: this.state.contextId,
          panelConfig: config.panels[this.state.contextId],
        }).ref(this.state.contextId)
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
