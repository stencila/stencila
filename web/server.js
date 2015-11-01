
var express = require('express');
var path = require('path');
var sass = require('node-sass');
var bodyParser = require('body-parser');

var app = express();
var port = process.env.PORT || 5000;
var browserify = require("browserify");

// use body parser so we can get info from POST and/or URL parameters
app.use(bodyParser.json({limit: '3mb'}));
app.use(bodyParser.urlencoded({ extended: true }));

// use static server
app.use(express.static(path.join(__dirname, "substance/app/assets")));
app.use('/ace', express.static(path.join(__dirname, "build/external/ace")));
app.use('/MathJax', express.static(path.join(__dirname, "build/external/MathJax")));
app.use('/data', express.static(path.join(__dirname, "substance/app/data")));
app.use('/i18n', express.static(path.join(__dirname, "substance/i18n")));

// Backend
// --------------------

app.get('/app.js', function (req, res, next) {
  // var startTime = Date.now();
  browserify({ debug: true, cache: false })
    .add(path.join(__dirname, "substance", "app", "app.js"))
    .bundle()
    .on('error', function(err){
      console.error(err.message);
      res.send('console.log("'+err.message+'");');
    })
    .pipe(res);
});

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

// use static server
app.use(express.static(__dirname));

app.get('/app.css', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.css);
  });
});

app.get('/app.css.map', function(req, res) {
  renderSass(function(err, result) {
    if (err) return handleError(err, res);
    res.set('Content-Type', 'text/css');
    res.send(result.map);
  });
});

app.listen(port, function(){
  console.log("Running at http://127.0.0.1:"+port+"/");
});

// Export app for requiring in test files
module.exports = app;
