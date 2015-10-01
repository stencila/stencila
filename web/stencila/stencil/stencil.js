var utilities = require('../utilities.js');
var Component = require('../component.js');

var Stencil = function(){
	var $content = $('#content');

	// Lazily load MathJax if there is math in the stencil
	var mathSelector = 'script[type^="math/tex"],script[type^="math/asciimath"]';
	if($content.find(mathSelector).length>0){
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
					$content.find(mathSelector).each(function(){
						$(this).css('display','none');
					});
					// Ensure these MathJax elements are not
					// editable when in Reveal mode. This needs to be done here
					// when we know these elements are present
					$content.find('.MathJax').attr('contenteditable','false');
				}
			);
		});
	}

	// Decorate figure and table captions with their number
	$content.find('table[data-index],figure[data-index]').each(function(){
		var item = $(this);
		var type = item.prop("tagName").toTitleCase();
		var index = item.attr('data-index');
		var caption = item.find('caption,figcaption');
		if(caption){
			caption.prepend('<span>'+type+' '+index+':</span>');
		}
	});
};

module.exports = {
	Stencil: Stencil
};

if(global.window){
	global.window.Stencila = {
		stencil: new Stencil()
	};
}
