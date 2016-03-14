'use strict';

var ControllerTool = require('substance/ui/ControllerTool');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

function HomeTool() {
  HomeTool.super.apply(this, arguments);
}
HomeTool.Prototype = function() {

  this.render = function() {
    var el = $$('div')
        .addClass('se-tool se-home-tool')
        .append(
            $$('a')
                .attr('href','https://stenci.la')
                .append(
                    $$('img')
                        .attr('src','/get/web/images/logo.svg')
                ),
            $$('span')
                .addClass('se-address')
                .text(this.props.address)
        )
    return el;
  };

};
ControllerTool.extend(HomeTool);

module.exports = HomeTool;
