/**
 * A development web server for the Stencila `web` module.
 *
 * Bundles Javascript and compiles SCSS on the fly so that a page refresh
 * can be used in development to load latest versions
 */

var express = require('express');
var proxy = require('express-http-proxy');
var url = require('url');
var path = require('path');
var sass = require('node-sass');
var browserify = require("browserify");

var handleError = function(err, res) {
  console.error(err);
  res.status(400).json(err);
};

var renderSass = function(cb) {
  sass.render({
    file: path.join(__dirname, 'stencil', 'stencil.scss'),
    sourceMap: true,
    outFile: 'app.css',
  }, cb);
};


var app = express();

// Home page
app.get('/', function(req, res){
  res.sendFile(path.join(__dirname, 'index.html'));
});

// Static components
app.use('/static', express.static(__dirname));

// Javascript
app.get('/get/web/stencil.min.js', function (req, res, next) {
  browserify({ debug: true, cache: false })
    .add(path.join(__dirname, 'stencil', 'stencil.js'))
    .bundle()
    .on('error', function(err){
      console.error(err);
    })
    .pipe(res);
});

// CSS
app.get('/get/web/stencil.min.css', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.css);
  });
});

// CSS map
app.get('/get/web/stencil.min.css.map', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.map);
  });
});

// Everything else at `/get/web` falls back to the `build` directory (e.g. fonts, MathJax)
// So, you'll need to do a build first
app.use('/get/web', express.static(path.join(__dirname, 'build')));

// Internationalization
app.use('/i18n', express.static(path.join(__dirname, "i18n")));

// Fallback to proxying to locally hosted components
// Don't use bodyParser middleware in association with this proxying,
// it seems to screw it up
app.use('*', proxy('localhost:7373', {
  forwardPath: function(req, res) {
    var uri = req.params[0];
    console.log('Proxying to http://localhost:7373'+uri);
    return url.parse(uri).path;
  },
}));

// Tell express no to set an Etag header
app.set('etag', false);

// Serve app
var port = process.env.PORT || 5000;
app.listen(port, function(){
  console.log("Running at http://127.0.0.1:"+port+"/");
});

// Export app for requiring in test files
module.exports = app;
