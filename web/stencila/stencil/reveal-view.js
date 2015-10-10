var utilities = require('../utilities.js');
var NormalView = require('./normal-view');

class RevealView extends NormalView {

	constructor(object){
		super(object);

		this.$root.addClass('reveal');
	}

	close(){
		this.$root.removeClass('reveal');
	}

	pull(){
		super.pull();
		var self = this;
		utilities.load("/web/external/ace/ace.js", function(){
			self.$root.find('[data-exec]').each(function(){
				var elem = $(this);
				// Add an id for ACE to work on
				var id = 'reveal-exec-' + new Date().getTime() + Math.floor(Math.random()*100000);
				elem.attr("id",id);
				// Create an Ace Editor instance in the dialog
				var editor = ace.edit(id);
				editor.setFontSize(14);
				var mode = {
					'cila': 'cila',
					'html': 'html',
					'js':   'javascript',
					'py':   'python',
					'r':    'r'						
				}['r'] || 'text';
				editor.getSession().setMode('ace/mode/'+mode);
				editor.setTheme("ace/theme/monokai");
				editor.setReadOnly(true);
				editor.setOption("minLines",1);
				editor.setOption("maxLines",50);
				editor.setShowPrintMargin(false);
				editor.setHighlightActiveLine(false);
				// Add padding before first and after last lines
				editor.renderer.setScrollMargin(5,5,0,0);
			});
		});
	}
}

module.exports = RevealView;
