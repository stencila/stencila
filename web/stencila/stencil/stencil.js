var Component = require('../component.js');

var Stencil = function(){

};

module.exports = {
	Stencil: Stencil
};

if(global.window){
	global.window.Stencila = {
		stencil: new Stencil()
	};
}
