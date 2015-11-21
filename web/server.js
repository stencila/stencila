/**
 * A development web server for the Stencila `web` module.
 *
 * Bundles Javascript and compiles SCSS on the fly so that a page refresh
 * can be used in development to load latest versions
 */

var express = require('express');
var path = require('path');
var sass = require('node-sass');
var bodyParser = require('body-parser');
var browserify = require("browserify");

var handleError = function(err, res) {
  console.error(err);
  res.status(400).json(err);
};

var renderSass = function(cb) {
  sass.render({
    file: path.join(__dirname, "substance", "app", "app.scss"),
    sourceMap: true,
    outFile: 'app.css',
  }, cb);
};


var app = express();
app.use(bodyParser.json({limit: '3mb'}));
app.use(bodyParser.urlencoded({ extended: true }));

// Static files
app.use(express.static(__dirname));
app.use('/get/web', express.static(path.join(__dirname, "build")));
app.use('/i18n', express.static(path.join(__dirname, "substance/i18n")));

// Javascript
app.get('/app.js', function (req, res, next) {
  browserify({ debug: true, cache: false })
    .add(path.join(__dirname, "substance", "app", "app.js"))
    .bundle()
    .on('error', function(err){
      handleError(err,res);
    })
    .pipe(res);
});

// CSS
app.get('/app.css', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.css);
  });
});

// CSS map
app.get('/app.css.map', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.map);
  });
});

// Serve app
var port = process.env.PORT || 5000;
app.listen(port, function(){
  console.log("Running at http://127.0.0.1:"+port+"/");
});

// Export app for requiring in test files
module.exports = app;
