window.Stencila = 1;
(function(){
	// Choose which "mode" to use (i.e which Javascript interface to load)..
	// First check if it is specified in the stencil
	var mode = document.querySelector('head meta[itemprop=mode]');
	if(mode) mode = mode.content;
	else mode = '';
	// If not specified in stencil then choose based on user agent
	if (mode.length === 0) {
		if(/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)) {
			mode = 'free';
		}
		else {
			mode = 'strict';
		}
	}
	document.write(unescape('%3Cscript src="'+(window.StencilaHost||'')+'/get/web/stencil-'+mode+'.min.js"%3E%3C/script%3E'));
	document.write(unescape('%3Clink href="'+(window.StencilaHost||'')+'/get/web/stencil-'+mode+'.min.css" rel="stylesheet" type="text/css" %3E%3C/link%3E'));
})();
