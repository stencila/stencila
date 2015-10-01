var Component = require('../component.js');

var Stencil = function(){
	var $content = $('#content');

	// Lazily load MathJax id there is math in the stencil
	var mathSelector = 'script[type^="math/tex"],script[type^="math/asciimath"]';
	if($content.find(mathSelector).length>0){
		// This is the recommended method for dynamically loading MathJax:
		//   https://docs.mathjax.org/en/latest/dynamic.html#loading-mathjax-dynamically
		// Previous attempts using ReqjuireJS and $.getSript worked some of the time but
		// had reliability issues (occaisional "Math Processing Error") probably related to timing
		(function () {
			var head = $('head');
			
			head.append(
				'<script type="text/x-mathjax-config">'+
					'MathJax.Hub.Config({' +
						'skipStartupTypeset: true,' +
						'showProcessingMessages: false,' +
						'showMathMenu: false,' +
						'"HTML-CSS": {preferredFont: "STIX"}' +
					'});' +
				'</script>'
			);

			var script = document.createElement("script");
			script.type = "text/javascript";
			script.src  = "/web/external/MathJax/MathJax.js?config=TeX-MML-AM_HTMLorMML";
			script.onload = function(){
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
			};
			head.get(0).appendChild(script);
		})();
	}
};

module.exports = {
	Stencil: Stencil
};

if(global.window){
	global.window.Stencila = {
		stencil: new Stencil()
	};
}
