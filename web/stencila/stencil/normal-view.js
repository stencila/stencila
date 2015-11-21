var utilities = require('../utilities.js');

class NormalView {

	constructor(stencil){
		var self = this;
		self.stencil = stencil;

		self.$el = $('#content');
		self.$el.addClass('normal-view');

		self.$el.on('click','button.refresh',function(){
			self.stencil.refresh();
		});

		self.$el.on('focus', function() {
			self.stencil.hold(self);
		});

		self.$el.on('input', function() {
			self.stencil.fling();
		});

		self.pull();
	}

	pull(){
		var self = this;
		self.stencil.html.then(function(html){
			self.$el.html(html);

			// Lazily load MathJax if there is math in the stencil
			var mathSelector = 'script[type^="math/tex"],script[type^="math/asciimath"]';
			if(self.$el.find(mathSelector).length>0){
				// This is the recommended method for dynamically loading MathJax:
				//   https://docs.mathjax.org/en/latest/dynamic.html#loading-mathjax-dynamically
				// Previous attempts using ReqjuireJS and $.getSript worked some of the time but
				// had reliability issues (occaisional "Math Processing Error") probably related to timing
				// Configure first...
				$('head').append(
					'<script type="text/x-mathjax-config">'+
						'MathJax.Hub.Config({' +
							'skipStartupTypeset: true,' +
							'showProcessingMessages: false,' +
							'showMathMenu: false,' +
							'"HTML-CSS": {preferredFont: "STIX"}' +
						'});' +
					'</script>'
				);
				// ...then load MathJax into head
				utilities.load("/get/web/external/MathJax/MathJax.js?config=TeX-MML-AM_HTMLorMML", function(){
					// Render using 'Rerender' instead of 'Typeset'
					// because math is already in <script type="math/..."> elements
					MathJax.Hub.Queue(
						["Rerender",MathJax.Hub,"content"],
						function(){
							// Hide math script elements which should now have been rendered into 
							// separate display elements by MathJax
							self.$el.find(mathSelector).each(function(){
								$(this).css('display','none');
							});
							// Ensure these MathJax elements are not
							// editable when in Reveal mode. This needs to be done here
							// when we know these elements are present
							self.$el.find('.MathJax').attr('contenteditable','false');
						}
					);
				});
			}
		});

		//self.$el.attr('contenteditable','true');
	}

	push(){
		// Create a temporary DOM to modify
		var dom = this.$el.clone();
		// Get all MathJax "jax" elements (e.g. 
		//    <script type="math/asciimath" id="MathJax-Element-2">e=m^2</script>
		// ) and remove the id if it starts with MathJax
		dom.find('script[type^="math/asciimath"],script[type^="math/tex"]').each(function(){
			var elem = $(this);
			if(/^MathJax/.exec(elem.attr('id'))) elem.removeAttr('id');
			// Remove the css style added above to hide these
			elem.removeAttr('style');
		});
		// Remove all elements which have been added
		dom.find('.MathJax_Error, .MathJax_Preview, .MathJax').remove();
		// Remove readonly atrributes added to inputs
		dom.find('input').each(function(){
			$(this).removeAttr('readonly');
		});

		this.stencil.html = dom.html();
	}

	inputs(){
		var inputs = {};
		this.$el.find('input').each(function(index,elem){
			elem = $(elem);
			inputs[elem.attr('name')] = elem.val();
		});
		return inputs;
	}
}

module.exports = NormalView;
