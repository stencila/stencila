var NormalView = require('./normal-view').NormalView;

class RevealView extends NormalView {

	constructor(object){
		super(object);

		this.$root.addClass('reveal');
	}

	close(){
		this.$root.removeClass('reveal');
	}

	edit(){
		this.$root.attr('contenteditable','true');
	}
}

module.exports = {
	RevealView: RevealView
};



