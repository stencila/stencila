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
var fs = require('fs');
var glob = require('glob');

var http = require('http');
var ws = require('ws');

var httpServer = http.createServer();
var wsServer = new ws.Server({ server: httpServer });


// Main application

var app = express();

// Paths specific to development

// Home page
app.get('/', function(req, res){
  res.sendFile(path.join(__dirname, 'index.html'));
});

// Tests
app.get('/tests', function (req, res, next) {
  res.send(
    '<!DOCTYPE html>\n<html><head></head><body>' + 
    'Yo! Open up a console (Ctrl+Shift+I) to see tests results. <script src="tests-bundle.js"></script>' + 
    '</body></html>'
  );
});
app.get('/tests-bundle.js', function (req, res, next) {
  browserify({
    debug: true,
    cache: false
  })
    .add(path.join(__dirname, 'tests/index.js'))
    .bundle()
    .on('error', function(err) {
      console.error(err.message);
      res.send('console.error("Browserify error: ' + err.message + '");');
    })
    .pipe(res);
});

// Test HTML pages are served up with the associated CSS and JS
// These pages are intended fo interactive testing of one node type
app.get('/tests/:type/*', function (req, res, next) {
  fs.readFile(path.join(__dirname, req.path, 'index.html'), "utf8", function (err, data) {
    if(err) return res.send(err);
    // Generate a simple page with necessary CSS and JS
    var type = req.params.type;
    var page = '<!DOCTYPE html>\n<html><head>';
    page += '<meta name="viewport" content="width=device-width, initial-scale=1">';
    page += '<link rel="stylesheet" type="text/css" href="/get/web/' + type + '.min.css">';
    page += '<script src="/get/web/' + type + '.min.js"></script>';
    page += '</head><body><main id="content">' + data + '</main></body></html>';
    res.set('Content-Type', 'text/html');
    res.send(page);
  })
});

// Examples
app.use('/examples', express.static(path.join(__dirname, "examples")));

// Paths that normally get served statically...

function nameToPath(name){
  var matches = name.match(/(\w+)-?(\w+)?/);
  var clas = matches[1];
  var mode = matches[2];
  var file = mode?(clas+'-'+mode):clas;
  return path.join(__dirname, clas, file);
}

// Javascript
app.get('/get/web/:name.min.js', function (req, res, next) {
  browserify({
    debug: true,
    cache: false
  })
    .add(nameToPath(req.params.name)+'.js')
    .bundle()
    .on('error', function(err) {
      console.error(err.message);
      res.send('console.error("Browserify error: ' + err.message + '");');
    })
    .pipe(res);
});

// CSS
function sassify(name, output, res) {
  sass.render({
    file: nameToPath(name)+'.scss',
    sourceMap: true,
    outFile: name+'.min.css',
  }, function(err, result){
    if (err) {
      console.error(err);
      res.status(500).json(err);
      return;
    }
    res.set('Content-Type', 'text/css');
    res.send(result[output]);
  });
}
app.get('/get/web/:name.min.css', function(req, res) {
  sassify(req.params.name, 'css', res);
});
app.get('/get/web/:name.min.css.map', function(req, res) {
  sassify(req.params.name, 'map', res);
});

// Images
app.use('/get/web/images', express.static(path.join(__dirname, 'images')));

// Everything else at `/get/web` falls back to the `build` directory (e.g. fonts, MathJax)
// So, you'll need to do a build first
app.use('/get/web', express.static(path.join(__dirname, 'build')));

// Internationalization
app.use('/i18n', express.static(path.join(__dirname, "i18n")));

// Deal with favicon to prevent uneeded error messages when no upstream proxy
app.get('/favicon.ico', function(req, res) {
  res.sendStatus(404);
});


// Collaboration server

var DocumentStore = require('./collab/DocumentStore');
var ChangeStore = require('./collab/ChangeStore');
var ModelFactory = require('./collab/ModelFactory');
var SnapshotEngine = require('./collab/SnapshotEngine');
var DocumentEngine = require('./collab/DocumentEngine');

var CollabServer = require('substance/collab/CollabServer');
var DocumentServer = require('substance/collab/DocumentServer');

var documentStore = new DocumentStore();
var changeStore = new ChangeStore();
var modelFactory = new ModelFactory();

var snapshotEngine = new SnapshotEngine({
  documentStore: documentStore,
  changeStore: changeStore,
  modelFactory: new ModelFactory()
});

var documentEngine = new DocumentEngine({
  documentStore: documentStore,
  changeStore: changeStore,
  snapshotEngine: snapshotEngine,
  modelFactory: modelFactory
});

var collabServer = new CollabServer({
  heartbeat: 30*1000,
  documentEngine: documentEngine
});
collabServer.bind(wsServer);

var documentServer = new DocumentServer({
  path: '/jam',
  documentEngine: documentEngine
});
documentServer.bind(app);



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
      if (!process.env.STENCILA_TOKEN) {
        console.error('Error no access token. Create an access token (e.g. at https://stenci.la/api/#!/Tokens/post_tokens) and copy its string into environment variable STENCILA_TOKEN');
        process.exit(1);
      } else {
        req.headers['Authorization'] = 'Token ' + process.env.STENCILA_TOKEN;
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



// Tell express not to set an Etag header
app.set('etag', false);

// Delegate http requests to express app
httpServer.on('request', app);

// Serve app
var host = 'localhost';
var port = process.env.PORT || 5000;
httpServer.listen(port, host, function(){
  console.log("Running at http://localhost:" + httpServer.address().port + "/");
});

// Export app for requiring in test files
module.exports = app;
