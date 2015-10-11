define(function(require, exports, module) {
	"use strict";

	var oop = require("../lib/oop");
	var TextMode = require("./text").Mode;
	var CilaHighlightRules = require("./cila_highlight_rules").CilaHighlightRules;
	var FoldMode = require("./folding/coffee").FoldMode;

	var Mode = function() {
	    this.HighlightRules = CilaHighlightRules;
	    this.foldingRules = new FoldMode();
	};
	oop.inherits(Mode, TextMode);

	(function() { 
	    this.$id = "ace/mode/cila";
	}).call(Mode.prototype);

	exports.Mode = Mode;
});
