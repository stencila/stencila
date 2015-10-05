var View = require('../view.js').View;

class RevealView extends View {

	constructor(object){
		super(object);

		$('#content').addClass('reveal');
	}

	close(){
		$('#content').removeClass('reveal');
	}
}

module.exports = {
	RevealView: RevealView
};



