var Directive = require('./directive');

class Exec extends Directive {

	constructor($el){
		super();
		this.lang = null;
		this.format = null;
		this.size = null;
		this.mode = null;
		this.show = false;

		this.code = null;

		if($el) this.pull($el);
	}

	pull($el){
		var attr = $el.attr('data-exec');
		var matches = attr.match(
			'(exec|cila|js|html|r|py)' +   // 1
			'(\\s+format\\s+([^\\s]+))?' + // 2-3
			'(\\s+size\\s+([^\\s]+))?' +   // 4-5
			'(\\s+(const|volat))?' +       // 6-7
			'(\\s+(show))?'                // 8-9
		);
		this.lang = matches[1];
		this.format = matches[3];
		this.size = matches[5];
		this.mode = matches[7];
		this.show = matches[9]==='show';

		this.code = $el.text();
	}

	push($el){
		var attr = this.lang || 'exec';
		if(this.format) attr += ' format ' + this.format;
		if(this.size) attr += ' size ' + this.size;
		if(this.mode!=='') attr += ' ' + this.mode;
		if(this.show) attr += ' show';
		$el.attr('data-exec',attr);

		$el.text(this.code);
	}

}

module.exports = Exec;
