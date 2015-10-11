var utilities = require('../utilities.js');

class CodeView {

	constructor(stencil,options,more){
		var self = this;
		
		this.stencil = stencil;

		var id = options.name + '-editor';
		this.$root = $(
			'<div class="' + options.name + '">' + 
				'<div id="' + id + '">' + 
				'</div>' + 
			'</div>'
		).appendTo('#main');

		utilities.load("/web/external/ace/ace.js", function(){
			var editor = self.editor = ace.edit(id);
			editor.getSession().setMode('ace/mode/' + options.mode);
			editor.setTheme("ace/theme/" + (options.theme || 'monokai'));

			editor.setFontSize(14);
			editor.setShowPrintMargin(false);
			// Set the maximum number of lines for the code. When the number
			// of lines exceeds this number a vertical scroll bar appears on the right
			editor.setOption("minLines",5);
			editor.setOption("maxLines",100000);
			// Set indented wrapped lines
			editor.setOptions({
	            wrap: true,
	            indentedSoftWrap: true,
	        });
			// Ensure that ACE passes on our special key bindings
            var f6 = 117, f7 = 118, f8 = 119, f9 = 120,
                r = 82;
            editor.commands.addCommand({
                name: 'render',
                bindKey: {win: 'Ctrl-R',  mac: 'Command-R'},
                exec: function(editor) {
                    $(document).trigger($.Event('keydown', { which: 82, ctrlKey: true} ));
                }
            });
            editor.commands.addCommand({
                name: 'F8',
                bindKey: {win: 'F8',  mac: 'F8'},
                exec: function(editor) {
                    $(document).trigger($.Event('keydown', { which: f8 }));
                }
            });
			// On focus, make this the master view
			editor.on('focus',function(){
				self.stencil.hold(self);
			});
			// On user change in content, fling to other views
			editor.on('change', function() {
				if(!self.silent) self.stencil.fling();
			});
			// More setup required by extending class
			if(more) more.call(self);
			// Now editor is set up pull content from stencil
			self.pull();
		});
	}

	close(){
		this.editor.destroy();
		this.$root.remove();
	}

	set(content){
		this.silent = true;
		this.editor.setValue(content);
		this.silent = false;
	}

	get(content){
		return this.editor.getValue();
	}
}

module.exports = CodeView;
