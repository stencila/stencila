var Context = require('./context.js');

var binary = require('node-pre-gyp');
var path = require('path');
var extension_path = binary.find(path.resolve(path.join(__dirname,'./package.json')));
var extension = require(extension_path);

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
