var utilities = require('../utilities.js');

var View = require('../view');

class NormalView extends View {

	constructor(object){
		super(object);
		var self = this;
		this.$root = $('#content');

		this.$root.on('click','button.refresh',function(){
			self.object.refresh();
		});
	}

	pull(){
		var self = this;
		self.object.html.then(function(html){
			self.$root.html(html);

			// Lazily load MathJax if there is math in the stencil
			var mathSelector = 'script[type^="math/tex"],script[type^="math/asciimath"]';
			if(self.$root.find(mathSelector).length>0){
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
				utilities.load("/web/external/MathJax/MathJax.js?config=TeX-MML-AM_HTMLorMML", function(){
					// Render using 'Rerender' instead of 'Typeset'
					// because math is already in <script type="math/..."> elements
					MathJax.Hub.Queue(
						["Rerender",MathJax.Hub,"content"],
						function(){
							// Hide math script elements which should now have been rendered into 
							// separate display elements by MathJax
							self.$root.find(mathSelector).each(function(){
								$(this).css('display','none');
							});
							// Ensure these MathJax elements are not
							// editable when in Reveal mode. This needs to be done here
							// when we know these elements are present
							self.$root.find('.MathJax').attr('contenteditable','false');
						}
					);
				});
			}
		});
	}

	inputs(){
		var inputs = {};
		this.$root.find('input').each(function(index,elem){
			elem = $(elem);
			inputs[elem.attr('name')] = elem.val();
		});
		return inputs;
	}
}

module.exports = NormalView;
