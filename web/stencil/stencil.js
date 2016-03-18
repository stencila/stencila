window.Stencila = 1;
(function(){
	var mode = document.querySelector('head meta[itemprop=mode]');
	if(mode) mode = mode.content;
	if(!mode){
		if(window.location.host.match('stenci.la|localhost')){
			if(/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)) {
				mode = 'free';
			}
			else {
				mode = 'strict';
			}
		} else {
			mode = 'free';
		}
	}
	document.write(unescape('%3Cscript src="'+(window.StencilaHost||'')+'/get/web/stencil-'+mode+'.min.js"%3E%3C/script%3E'));
	document.write(unescape('%3Clink href="'+(window.StencilaHost||'')+'/get/web/stencil-'+mode+'.min.css" rel="stylesheet" type="text/css" %3E%3C/link%3E'));
})();
