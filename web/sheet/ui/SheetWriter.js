'use strict';

var SheetController = require('./SheetController');
var SplitPane = require('substance-fe0ed/ui/SplitPane');
var ScrollPane = require('substance-fe0ed/ui/ScrollPane');
var StatusBar = require('substance-fe0ed/ui/StatusBar');
var Toolbar = require('substance-fe0ed/ui/Toolbar');
var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;
var Icon = require('substance-fe0ed/ui/FontAwesomeIcon');

var UndoTool = require('substance-fe0ed/ui/UndoTool');
var RedoTool = require('substance-fe0ed/ui/RedoTool');

var HomeTool = require('../../shared/tools/home/HomeTool');
var SaveTool = require('./tools/SaveTool');
var CommitTool = require('./tools/CommitTool');
var DisplayModeTool = require('./tools/DisplayModeTool');

var SheetEditor = require('./SheetEditor');
var Sheet = require('../model/Sheet');

var CONFIG = {
  controller: {
    commands: [
      require('substance-fe0ed/ui/UndoCommand'),
      require('substance-fe0ed/ui/RedoCommand'),
      require('./commands/SaveCommand'),
      require('./commands/CommitCommand')
    ],
    components: {
      // Registry for different cell content types
      'boolean': require('./Boolean'),
      'integer': require('./PrimitiveExpression'),
      'real': require('./PrimitiveExpression'),
      'string': require('./PrimitiveExpression'),
      'html': require('./HTMLCellComponent'),
      'image_file': require('./ImageExpression'),
      'error': require('./Error')
    }
  },
  main: {
    commands: [
      /* Not used ATM
      // Special commands
      require('substance-fe0ed'/packages/embed/EmbedCommand'),
      require('substance-fe0ed'/packages/strong/StrongCommand'),
      require('substance-fe0ed'/packages/emphasis/EmphasisCommand'),
      require('substance-fe0ed'/packages/link/LinkCommand'),
      require('substance-fe0ed'/packages/subscript/SubscriptCommand'),
      require('substance-fe0ed'/packages/superscript/SuperscriptCommand'),
      require('substance-fe0ed'/packages/code/CodeCommand')
      */
    ],
    textTypes: [
    ]
  }
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

  /*
    Called when the table selection is changed so we can
    update the display mode tool accordingly.
  */
  this._onSelectionChanged = function(sel) {
    var displayModeTool = this.refs.displayModeTool;
    var doc = this.props.doc;
    var cell;
    if (sel.isCollapsed) {
      cell = doc.getCellAt(sel.startRow, sel.startCol);
    }
    if (cell) {
      displayModeTool.setState({
        disabled: false,
        cell: cell
      });
      return;
    } else {
      displayModeTool.setState({
        disabled: true,
        cell: null
      });
    }
  };

  this._renderMainSection = function() {
    return $$('div').ref('main').addClass('se-main-section').append(
      $$(SplitPane, {splitType: 'horizontal'}).append(
        // Menu Pane on top
        $$(Toolbar).ref('toolbar').append(
          $$(Toolbar.Group).append(
            $$(HomeTool, {
              address: this.props.engine.address
            }).ref('homeTool')
          ),
          $$(Toolbar.Group).addClass('float-right').append(
            $$(UndoTool).append($$(Icon, {icon: 'fa-undo'})),
            $$(RedoTool).append($$(Icon, {icon: 'fa-repeat'})),
            // Removed for now, see #132
            //$$(SaveTool).append($$(Icon, {icon: 'fa-save'})),
            //$$(CommitTool)
            $$(DisplayModeTool).ref('displayModeTool')
          )
        ),
        // Content Panel below
        $$(ScrollPane, {
          scrollbarPosition: 'left'
        }).ref('contentPanel').append(
          $$('div').ref('main').addClass('document-content').append(
            $$(SheetEditor, {
              mode: this.props.mode,
              doc: this.props.doc,
              onSelectionChanged: this._onSelectionChanged.bind(this)
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
