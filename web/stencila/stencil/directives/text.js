var Directive = require('./directive');

class Text extends Directive {

	constructor($el){
		super();
		this.expr = null;

		if($el) this.pull($el);
	}

	pull($el){
		this.expr = $el.attr('data-text');
	}

	push($el){
		$el.attr('data-text',this.expr);
	}

}

module.exports = Text;
