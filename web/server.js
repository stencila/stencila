/**
 * A development web server for the Stencila `web` module.
 *
 * Bundles Javascript and compiles SCSS on the fly so that a page refresh
 * can be used in development to load latest versions. Proxies other
 * requests to a Stencila component host (e.g. one started locally using `stencila-r serve`)
 *
 * Usage:
 * 
 *   node server.js
 *
 * Or, to proxy to a host other than http://localhost:7373 add the host's URL. e.g.
 * 
 *   node server.js https://stenci.la
 */

var express = require('express');
var proxy = require('express-http-proxy');
var url = require('url');
var path = require('path');
var sass = require('node-sass');
var browserify = require("browserify");

var handleBrowserifyError = function(err, res) {
  console.error(err.message);
  //This crashes server for some strange reason, so commented out
  //res.send('console.log("Browserify error '+err.message+'");');
};

var handleSassError = function(err, res) {
  console.error(err);
  res.status(400).json(err);
};

var renderSass = function(type,cb) {
  sass.render({
    file: path.join(__dirname, type, type+'.scss'),
    sourceMap: true,
    outFile: type+'.min.css',
  }, cb);
};


var app = express();

// Home page
app.get('/', function(req, res){
  res.sendFile(path.join(__dirname, 'index.html'));
});

// Example components
app.use('/examples', express.static(path.join(__dirname, "examples")));

// Javascript
app.get('/get/web/:type.min.js', function (req, res, next) {
  browserify({ debug: true, cache: false })
    .add(path.join(__dirname, req.params.type, req.params.type+'.js'))
    .bundle()
    .on('error', function(err){
      handleBrowserifyError(err);
    })
    .pipe(res);
});

// CSS
app.get('/get/web/:type.min.css', function(req, res) {
  renderSass(req.params.type,function(err, result) {
    if (err) return handleSassError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.css);
  });
});

// CSS map
app.get('/get/web/:type.min.css.map', function(req, res) {
  renderSass(req.params.type,function(err, result) {
    if (err) return handleSassError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.map);
  });
});

// Everything else at `/get/web` falls back to the `build` directory (e.g. fonts, MathJax)
// So, you'll need to do a build first
app.use('/get/web', express.static(path.join(__dirname, 'build')));

// Internationalization
app.use('/i18n', express.static(path.join(__dirname, "i18n")));

// Fallback to proxying to hosted components
// Don't use bodyParser middleware in association with this proxying,
// it seems to screw it up
var upstream = 'http://localhost:7373';
if (process.argv[2]){
  upstream = process.argv[2];
}
app.use('*', proxy(upstream, {
  decorateRequest: function(req) {
    if(upstream!=='http://localhost:7373') {
      if (!process.env.STENCILA_HUB_TOKEN) {
        console.error('Error no access token. Create an access token (e.g. at https://stenci.la/api/#!/Tokens/post_tokens) and copy its string into environment variable STENCILA_HUB_TOKEN');
        process.exit(1);
      } else {
        req.headers['Authorization'] = 'Token ' + process.env.STENCILA_HUB_TOKEN;
      }
    }
    return req;
  },
  forwardPath: function(req, res) {
    var uri = req.params[0];
    console.log('Proxying to ' + upstream + uri);
    return url.parse(uri).path;
  },
}));

// Tell express no to set an Etag header
app.set('etag', false);

// Serve app
var port = process.env.PORT || 5000;
app.listen(port, function(){
  console.log("Running at http://localhost:"+port+"/");
});

// Export app for requiring in test files
module.exports = app;
