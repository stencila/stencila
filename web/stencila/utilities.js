/**
 * Augmentation of string prototype
 */
String.prototype.toTitleCase = function () {
    return this.replace(/\w\S*/g, function(txt){return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase();});
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
	load: load
};
