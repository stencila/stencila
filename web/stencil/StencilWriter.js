'use strict';

var oo = require('substance-fe0ed/util/oo');
var StencilController = require('./StencilController');
var docHelpers = require('substance-fe0ed/model/documentHelpers');
var WriterTools = require('./packages/writer/WriterTools');
var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;

var CONFIG = {
  controller: {
    commands: [
      require('substance-fe0ed/ui/UndoCommand'),
      require('substance-fe0ed/ui/RedoCommand'),
      require('substance-fe0ed/ui/SaveCommand'),

      require('./packages/writer/RenderCommand'),
    ],
    components: {
      // Substance components, in alphabetical order, from `substance/packages` 
      // Note that not all Substance packages have components so they are not all listed here
      'blockquote': require('substance-fe0ed/packages/blockquote/BlockquoteComponent'),
      //'codeblock': require('substance-fe0ed/packages/codeblock/CodeblockComponent'),
      'heading': require('substance-fe0ed/packages/heading/HeadingComponent'),
      'image': require('substance-fe0ed/packages/image/ImageComponent'),
      'link': require('substance-fe0ed/packages/link/LinkComponent'),
      // These imports currently fail
      //'list': require('substance-fe0ed/packages/list/ListComponent'),
      //'list-item': require('substance-fe0ed/packages/list/ListItemComponent'),
      'paragraph': require('substance-fe0ed/packages/paragraph/ParagraphComponent'),
      'table': require('substance-fe0ed/packages/table/TableComponent'),

      // Stencil components
      'stencil-title': require('./packages/title/StencilTitleComponent'),
      'stencil-summary': require('./packages/summary/StencilSummaryComponent'),

      'stencil-math': require('./packages/math/StencilMathComponent'),
      'stencil-equation': require('./packages/equation/StencilEquationComponent'),
      'stencil-codeblock': require('./packages/codeblock/StencilCodeblockComponent'),

      'stencil-parameter': require('./packages/parameter/StencilParameterComponent'),
      'stencil-exec': require('./packages/exec/StencilExecComponent'),
      'stencil-out': require('./packages/out/StencilOutComponent'),
      'stencil-figure': require('./packages/figure/StencilFigureComponent'),
      'stencil-include': require('./packages/include/StencilIncludeComponent'),
      'stencil-text': require('./packages/text/StencilTextComponent'),

      'stencil-default-node': require('./packages/default/StencilDefaultNodeComponent'),

      // Panels
      'toc': require('substance-fe0ed/ui/TOCPanel'),
      //'cila': require('./packages/writer/CilaPanel'),
      'edit-source': require('./packages/writer/EditSourcePanel')
    }
  },
  body: {
    commands: [
      require('substance-fe0ed/packages/embed/EmbedCommand'),
      require('substance-fe0ed/packages/text/SwitchTextTypeCommand'),
      require('substance-fe0ed/packages/strong/StrongCommand'),
      require('substance-fe0ed/packages/emphasis/EmphasisCommand'),
      require('substance-fe0ed/packages/link/LinkCommand'),
      require('substance-fe0ed/packages/subscript/SubscriptCommand'),
      require('substance-fe0ed/packages/superscript/SuperscriptCommand'),
      require('substance-fe0ed/packages/code/CodeCommand'),

      require('./packages/exec/StencilExecInsertCommand'),
      require('./packages/text/StencilTextInsertCommand'),
      require('./packages/figure/StencilFigureInsertCommand'),
      require('./packages/include/StencilIncludeInsertCommand'),
    ],
    textTypes: [
      {name: 'paragraph', data: {type: 'paragraph'}},
      {name: 'heading1',  data: {type: 'heading', level: 1}},
      {name: 'heading2',  data: {type: 'heading', level: 2}},
      {name: 'heading3',  data: {type: 'heading', level: 3}},
      //{name: 'codeblock', data: {type: 'codeblock'}},
      {name: 'blockquote', data: {type: 'blockquote'}},

      {name: 'title', data: {type: 'stencil-title'}},
      {name: 'summary', data: {type: 'stencil-summary'}},
    ]
  },
  panels: {
    'toc': {
      hideContextToggles: false
    },
    /*'cila': {
      hideContextToggles: false
    },*/
    'edit-source': {
      hideContextToggles: true
    }
  },
  tabOrder: ['toc',/*'cila'*/],
  containerId: 'body',
  isEditable: true
};

function StencilWriter() {
  StencilWriter.super.apply(this, arguments);
}

StencilWriter.Prototype = function() {

  var _super = Object.getPrototypeOf(this);

  this._renderToolbar = function() {
    return _super._renderToolbar.call(this).append(
      $$(WriterTools,{
        engine: this.props.engine
      })
    );
  };

  this.render = function() {
    return _super.render.call(this)
      .addClass('sc-stencil-writer');
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
};

oo.inherit(StencilWriter, StencilController);

StencilWriter.static.config = CONFIG;

module.exports = StencilWriter;
