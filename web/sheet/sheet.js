'use strict';

var Raven = require('raven-js');

var Component = require('substance-fe0ed/ui/Component');
var $ = window.$ = require('substance-fe0ed/util/jquery');
var SheetWriter = require('./ui/SheetWriter');
var SheetHTMLImporter = require('./model/SheetHTMLImporter');
var SheetHTMLExporter = require('./model/SheetHTMLExporter');

var SheetRemoteEngine = require('./engine/SheetRemoteEngine');
var engine = new SheetRemoteEngine();

var utilities = require('../shared/utilities');

// Required to indicate succesful loading of the JS
// and that a fallback not required. Also used to hold
// references to sheet and other debugging usefullnesses
window.Stencila = {};

/**
 * Render a static version of the sheet document
 * 
 * This generates the HTML for the sheet. Eventually this may
 * be done server side. Currently HTML is generated server side
 * but using the backend `Sheet::page()` method.
 */
function renderStatic(doc) {
  var ghostEl = document.createElement('div');
  var writer = Component.mount(SheetWriter, {
    mode: 'read',
    doc: doc,
    engine: engine
  }, ghostEl);
  document.body.innerHTML = ghostEl.innerHTML;
  return writer;
}

/**
 * Render a dynamic version of the sheet document
 * 
 * A dynamic version allows for cell updating etc
 */
function renderDynamic(doc, mode) {
  document.body.innerHTML = '';
  var writer = Component.mount(SheetWriter, {
    mode: mode,
    doc: doc,
    engine: engine
  }, document.body);
  return writer;
}

/**
 * Load the sheet
 */
function load() {
  // Create sheet document from page HTML
  var content = $('#content');
  var html = content.html() || '';
  document.body.innerHTML = '';
  var importer = new SheetHTMLImporter();
  var doc = importer.importDocument(html);

  // Render the sheet document
  var writer = renderStatic(doc);
  engine.boot(function(err) {
    var logger = writer.getLogger();
    if (err) {
      return logger.error(err.message || err.toString());
    }
    if (this.active) {
      writer = renderDynamic(doc, 'write');
    }
    logger.info('Booted with rights: ' + this.rights.action || this.rights);
  }.bind(engine));

  // Asychronously load MathJax (it can't be required) and render
  // script elements
  window.MathJax = {
    skipStartupTypeset: true,
    showProcessingMessages: false,
    showMathMenu: false,
    "HTML-CSS": {preferredFont: "STIX"}
  };
  utilities.load('/get/web/mathjax/MathJax.js?config=TeX-MML-AM_HTMLorMML', function() {
    MathJax.Hub.Queue(
      ["Rerender",MathJax.Hub,"content"]
    );
  });

  // Expose doc and engine for debugging in the console
  window.Stencila.doc = doc;
  window.Stencila.writer = writer;
  window.Stencila.engine = engine;
}

/**
 * Startup the app
 */
function startup() {
  var doc;
  if (window.location.hostname.match(/localhost|(127\.0\.0\.1)/)) {
    load();
  } else {
    Raven
      .config('https://6329017160394100b21be92165555d72@app.getsentry.com/37250')
      .install();
    try {
      load();
    } catch(e) {
      Raven.captureException(e)
    }
  }
}

/**
 * Shutdown the app
 */
function shutdown() {
  engine.shutdown();
}


// Run startup when document ready
$(startup);
// Run shutdown when page is unloaded
window.onunload = function() {
  shutdown();
}
