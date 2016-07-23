'use strict';

var Toolbar = require('substance-fe0ed/ui/Toolbar');
var Component = require('substance-fe0ed/ui/Component');
var $$ = Component.$$;

var Icon = require('substance-fe0ed/ui/FontAwesomeIcon');

var SwitchTextTypeTool = require('substance-fe0ed/packages/text/SwitchTextTypeTool');
var UndoTool = require('substance-fe0ed/ui/UndoTool');
var RedoTool = require('substance-fe0ed/ui/RedoTool');
var SaveTool = require('substance-fe0ed/ui/SaveTool');
var StrongTool = require('substance-fe0ed/packages/strong/StrongTool');
var EmphasisTool = require('substance-fe0ed/packages/emphasis/EmphasisTool');
var LinkTool = require('substance-fe0ed/packages/link/LinkTool');

var HomeTool = require('../../../shared/tools/home/HomeTool');
var ActivateTool = require('./ActivateTool');
var RenderTool = require('./RenderTool');

var StencilExecInsertTool = require('../exec/StencilExecInsertTool');
var StencilTextInsertTool = require('../text/StencilTextInsertTool');
var StencilFigureInsertTool = require('../figure/StencilFigureInsertTool');
var StencilIncludeInsertTool = require('../include/StencilIncludeInsertTool');

function WriterTools() {
  WriterTools.super.apply(this, arguments);
}
WriterTools.Prototype = function() {
  this.render = function() {
    return $$('div').append(
      $$(Toolbar.Group).append(
        $$(HomeTool, {
          address: this.props.engine.address
        })
      ),
      $$(Toolbar.Group).addClass('float-right').append(
        $$(SwitchTextTypeTool)
      ),
      $$(Toolbar.Group).addClass('float-right').append(
        $$(UndoTool).append($$(Icon, {icon: 'fa-undo'})),
        $$(RedoTool).append($$(Icon, {icon: 'fa-repeat'}))
      ),
      $$(Toolbar.Group).addClass('float-right').append(
        $$(RenderTool).append($$(Icon, {icon: 'fa-refresh'})),
        $$(SaveTool).append($$(Icon, {icon: 'fa-save'}))
      ),
      $$(Toolbar.Dropdown, {label: $$(Icon, {icon: 'fa-plus'}),}).append(
        $$(StencilExecInsertTool).append($$(Icon, {icon: 'fa-play'}),' execute'),
        $$(StencilTextInsertTool).append($$(Icon, {icon: 'fa-font'}),' text'),
        $$(StencilFigureInsertTool).append($$(Icon, {icon: 'fa-bar-chart'}),' figure'),
        $$(StencilIncludeInsertTool).append($$(Icon, {icon: 'fa-arrow-circle-right'}),' include')
      ),
      $$(Toolbar.Group).addClass('float-right').append(
        $$(StrongTool).append($$(Icon, {icon: 'fa-bold'})),
        $$(EmphasisTool).append($$(Icon, {icon: 'fa-italic'})),
        $$(LinkTool).append($$(Icon, {icon: 'fa-link'}))
      )
    );
  };
};
Component.extend(WriterTools);
module.exports = WriterTools;
