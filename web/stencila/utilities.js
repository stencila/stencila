/**
 * Augmentation of string prototype
 */
String.prototype.toTitleCase = function () {
    return this.replace(/\w\S*/g, function(txt){return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase();});
};

var query = {
	/**
	 * Get query string parameter
	 * Thanks to http://stackoverflow.com/a/901144/4625911
	 */
	param : function(name) {
	    name = name.replace(/[\[]/, "\\[").replace(/[\]]/, "\\]");
	    var regex = new RegExp("[\\?&]" + name + "=([^&#]*)"),
	        results = regex.exec(location.search);
	    return results === null ? "" : decodeURIComponent(results[1].replace(/\+/g, " "));
	}
};

/**
 * Load a script from the current host of Stencila
 * Javascript and CSS
 */
function load(source,callback){
	var head = document.getElementsByTagName("head")[0];
	var script = document.createElement("script");
	script.type = "text/javascript";
	script.src  = (window.StencilaHost || '') + source;
	if(callback) script.onload = callback;
	head.appendChild(script);
}

module.exports = {
	query: query,
	load: load
};
