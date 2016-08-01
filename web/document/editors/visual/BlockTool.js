var SwitchTextTypeTool = require('substance/packages/base/SwitchTextTypeTool');

function BlockTool() {
  BlockTool.super.apply(this, arguments);
}

BlockTool.Prototype = function() {

	var _super = BlockTool.super.prototype;

	this.render = function($$) {
		var el = _super.render.call(this, $$);
		el.addClass('sc-block-tool');
		return el;
	}
};

SwitchTextTypeTool.extend(BlockTool);

module.exports = BlockTool;
