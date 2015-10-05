var View = require('../view.js').View;

class HtmlView extends View {

	constructor(object){
		super(object);

		this._root = $('#main').append('<div class="html-view"></div>');
	}

	update(){
		this._object.html.then(function(html){
		});
	}
}

module.exports = {
	HtmlView: HtmlView
};



