/**
 * Bundle `vendor/jupyter-kernel.min.js` from `node_modules/jupyter-notebook-deps/notebook/static/services/kernels/kernel.js`
 * 
 * Based on `node_modules/jupyter-notebook-deps/notebook/tools/build-main.js` but simplified
 * to deal with one modues only
 */

var rjs = require('requirejs').optimize;

var rjs_config = {
  baseUrl: 'node_modules/jupyter-notebook-deps/notebook/static',
  
  name: 'services/kernels/kernel',
  out: 'vendor/jupyter-kernel.min.js',
  
  generateSourceMaps: false,

  // These paths determined by trial and error to be a minimum set
  // for succesful compilation
  paths: {
    underscore : 'components/underscore/underscore-min',
    jquery: 'components/jquery/jquery.min',
    moment: 'components/moment/moment',
    codemirror: 'components/codemirror'
  }
};

rjs(rjs_config, console.log, function (err) {
  console.log("Failed to build", err);
  process.exit(1);
});
