'use strict';

var SheetController = require('./SheetController');
var SplitPane = require('substance/ui/SplitPane');
var ScrollPane = require('substance/ui/ScrollPane');
var Toolbar = require('substance/ui/Toolbar');
var Component = require('substance/ui/Component');

var $$ = Component.$$;
var UndoTool = require('substance/ui/UndoTool');
var RedoTool = require('substance/ui/RedoTool');
var SaveTool = require('substance/ui/SaveTool');
var StrongTool = require('substance/packages/strong/StrongTool');
var CodeTool = require('substance/packages/code/CodeTool');
var EmphasisTool = require('substance/packages/emphasis/EmphasisTool');
var Icon = require('substance/ui/FontAwesomeIcon');
var LinkTool = require('substance/packages/link/LinkTool');
var SheetEditor = require('./SheetEditor');
var Sheet = require('../model/Sheet');

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand')
    ],
    components: {

    },
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
    'updateCell': this.onUpdateCell
  });
}

SheetWriter.Prototype = function() {

  this.onUpdateCell = function(cell, sheet) {
    // Update the sheet with the new cell source
    this.props.engine.update([{
      "id" : cell.cid,
      "source" : cell.source
    }], function(error, updates) {
      for(var index = 0; index < updates.length; index++) {
        var update = updates[index];
        var coords = Sheet.static.getRowCol(update.id);
        var cellComponent = sheet.getCellAt(coords[0], coords[1]);
        var cellNode = cellComponent.getNode();
        cellNode.tipe = update.type;
        cellNode.value = update.value;
        cellComponent.rerender();
      }
    });
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
          // scrollbarType: 'substance',
          scrollbarPosition: 'left'
        }).ref('contentPanel').append(
          // The full fledged document (ContainerEditor)
          $$('div').ref('main').addClass('document-content').append(
            $$(SheetEditor, {
              doc: this.props.doc
            }).ref('sheetEditor')
          )
        )
      ).ref('mainSectionSplitPane')
    );
  };

  this.render = function() {
    return $$('div').addClass('sc-sheet-controller sc-sheet-writer sc-controller').append(
      this._renderMainSection()
    );
  };
};

SheetController.extend(SheetWriter);
SheetWriter.static.config = CONFIG;

module.exports = SheetWriter;