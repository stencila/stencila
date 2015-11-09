var Toolbar = require('substance/ui/Toolbar');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var SwitchTextTypeTool = require('substance/packages/text/SwitchTextTypeTool');
var UndoTool = require('substance/ui/UndoTool');
var RedoTool = require('substance/ui/RedoTool');
var SaveTool = require('substance/ui/SaveTool');
var RenderTool = require('./RenderTool');
var StrongTool = require('substance/packages/strong/StrongTool');
var EmphasisTool = require('substance/packages/emphasis/EmphasisTool');
var Icon = require('substance/ui/FontAwesomeIcon');
var LinkTool = require('substance/packages/link/LinkTool');

var WriterTools = Component.extend({
  render: function() {
    return $$('div').append(
      $$(Toolbar.Group).append(
        $$(SwitchTextTypeTool)
      ),
      $$(Toolbar.Group).append(
        $$(UndoTool).append($$(Icon, {icon: 'fa-undo'})),
        $$(RedoTool).append($$(Icon, {icon: 'fa-repeat'})),
        $$(SaveTool).append($$(Icon, {icon: 'fa-save'})),
        $$(RenderTool).append($$(Icon, {icon: 'fa-flash'}))
      ),
      $$(Toolbar.Dropdown, {label: $$(Icon, {icon: 'fa-image'}),}).append(
        $$(StrongTool).append($$(Icon, {icon: 'fa-bold'}))
      ),
      $$(Toolbar.Group).addClass('float-right').append(
        $$(StrongTool).append($$(Icon, {icon: 'fa-bold'})),
        $$(EmphasisTool).append($$(Icon, {icon: 'fa-italic'})),
        $$(LinkTool).append($$(Icon, {icon: 'fa-link'}))
      )
    );
  }
});

module.exports = WriterTools;