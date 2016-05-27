var Context = require('./context.js');

var extension = require('./build/Release/extension');

var Stencil = function(){
  var instance = extension.Stencil();
  var context = new Context();
  instance.attach(context);
  return instance;
};

module.exports = {
  Context: Context,
  Stencil: Stencil,
  Sheet: extension.Sheet
};
