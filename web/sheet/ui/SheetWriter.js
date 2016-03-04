'use strict';

var SheetController = require('./SheetController');
var SplitPane = require('substance/ui/SplitPane');
var ScrollPane = require('substance/ui/ScrollPane');
var StatusBar = require('substance/ui/StatusBar');
var Toolbar = require('substance/ui/Toolbar');
var Component = require('substance/ui/Component');

var $$ = Component.$$;
var UndoTool = require('substance/ui/UndoTool');
var RedoTool = require('substance/ui/RedoTool');
var StrongTool = require('substance/packages/strong/StrongTool');
var CodeTool = require('substance/packages/code/CodeTool');
var EmphasisTool = require('substance/packages/emphasis/EmphasisTool');
var Icon = require('substance/ui/FontAwesomeIcon');
var LinkTool = require('substance/packages/link/LinkTool');
var SaveTool = require('./SaveTool');
var SheetEditor = require('./SheetEditor');
var Sheet = require('../model/Sheet');

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('./SaveCommand')
    ],
    components: {
      // Registry for different cell content types
      'string': require('./PrimitiveExpression'),
      'integer': require('./PrimitiveExpression'),
      'real': require('./PrimitiveExpression'),
      'ImageFile': require('./ImageExpression'),
      'error': require('./Error')
    }
  },
  main: {
    commands: [
      // Special commands
      require('substance/packages/embed/EmbedCommand'),
      require('substance/packages/strong/StrongCommand'),
      require('substance/packages/emphasis/EmphasisCommand'),
      require('substance/packages/link/LinkCommand'),
      require('substance/packages/subscript/SubscriptCommand'),
      require('substance/packages/superscript/SuperscriptCommand'),
      require('substance/packages/code/CodeCommand')
    ],
    textTypes: [
    ]
  },
  isEditable: true
};

function SheetWriter(parent, params) {
  SheetController.call(this, parent, params);

  this.handleActions({
    'updateCells': this.updateCells
  });
}

SheetWriter.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-sheet-controller sc-sheet-writer sc-controller').append(
      $$(SplitPane, {splitType: 'horizontal', sizeB: 'inherit'}).append(
        this._renderMainSection(),
        $$(StatusBar, {doc: this.props.doc}).ref('statusBar')
      ).ref('workspaceSplitPane')
    );
    return el;
  };

  this._renderMainSection = function() {
    return $$('div').ref('main').addClass('se-main-section').append(
      $$(SplitPane, {splitType: 'horizontal'}).append(
        // Menu Pane on top
        $$(Toolbar).ref('toolbar').append(
          $$(Toolbar.Group).append(
            $$(UndoTool).append($$(Icon, {icon: 'fa-undo'})),
            $$(RedoTool).append($$(Icon, {icon: 'fa-repeat'})),
            $$(SaveTool).append($$(Icon, {icon: 'fa-save'}))
          ),
          $$(Toolbar.Group).addClass('float-right').append(
            $$(StrongTool).append($$(Icon, {icon: 'fa-bold'})),
            $$(EmphasisTool).append($$(Icon, {icon: 'fa-italic'})),
            $$(LinkTool).append($$(Icon, {icon: 'fa-link'})),
            $$(CodeTool).append($$(Icon, {icon: 'fa-code'}))
          )
        ),
        // Content Panel below
        $$(ScrollPane, {
          scrollbarPosition: 'left'
        }).ref('contentPanel').append(
          $$('div').ref('main').addClass('document-content').append(
            $$(SheetEditor, {
              mode: this.props.mode,
              doc: this.props.doc
            }).ref('sheetEditor')
          )
        )
      ).ref('mainSectionSplitPane')
    );
  };

  this.updateCells = function(cells) {
    cells = cells.map(function(cell) {
      return {
        id: Sheet.static.getCellId(cell.row, cell.col),
        source: cell.content || '',
        display: cell.displayMode
      };
    });
    // Update the sheet with the new cell source
    this.props.engine.update(cells, function(err, updates) {
      if (err) {
        this.getLogger().error(err.message || err.toString());
        return;
      }
      if (!updates) {
        console.error('FIXME: did not receive updates.', updates);
        return;
      }
      this._handleUpdates(updates);
    }.bind(this));
  };

  this._handleUpdates = function(updates) {
    var sheet = this.props.doc;
    for(var index = 0; index < updates.length; index++) {
      var update = updates[index];
      var coords = Sheet.static.getRowCol(update.id);
      var cell = sheet.getCellAt(coords[0], coords[1]);
      if (cell) {
        cell.kind = update.kind;
        cell.valueType = update.type;
        cell.value = update.value;
        cell.displayMode = update.display;
        console.log('updated cell', cell);
        cell.emit('cell:changed');
      }
    }
    if (updates.length > 0) {
      this.refs.sheetEditor._rerenderSelection();
    }
  };
};

SheetController.extend(SheetWriter);
SheetWriter.static.config = CONFIG;

module.exports = SheetWriter;
