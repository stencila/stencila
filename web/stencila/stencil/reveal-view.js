var utilities = require('../utilities.js');
var NormalView = require('./normal-view');

var Exec = require('./directives/exec');
var Text = require('./directives/text');

class RevealView extends NormalView {

	constructor(stencil){
		super(stencil);

		this.$el.addClass('reveal');
	}

	close(){
		this.$el.removeClass('reveal');
	}

	pull(){
		super.pull();
		var self = this;
		utilities.load("/get/web/external/ace/ace.js", function(){

			self.$el.find('[data-exec]').each(function(){
				var $el = $(this);
				// Create a new Exec directive
				var dir = new Exec($el);

				// Create editor
				// Add an id to elem for ACE to work on
				var id = 'reveal-exec-' + 
					new Date().getTime() + 
					Math.floor(Math.random()*100000);
				$el.attr("id",id);
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

			self.$el.find('[data-text]').each(function(){
				var $el = $(this);
				var dir = new Text($el);
			});

		});
	}
}

module.exports = RevealView;
