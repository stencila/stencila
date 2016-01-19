'use strict';

var Component = require('substance/ui/Component');
var $ = window.$ = require('substance/util/jquery');
var SheetWriter = require('./ui/SheetWriter');
var SheetHTMLImporter = require('./model/SheetHTMLImporter');
var SheetHTMLExporter = require('./model/SheetHTMLExporter');

var SheetRemoteEngine = require('./engine/SheetRemoteEngine');
var engine = new SheetRemoteEngine();

function loadDocument() {
  var content = $('#content');
  var html = content.html() || '';
  document.body.innerHTML = '';

  var importer = new SheetHTMLImporter();
  var doc = importer.importDocument(html);
  return doc;
}

function activateSession(cb) {
  console.log('activating session');
  engine.activate(function(err, res) {
    console.log('activate.res', res);
    cb();
  });
}

// At some point this may run on the server
function renderStaticReadonlyVersion(doc) {
  var ghostEl = document.createElement('div');
  Component.mount(SheetWriter, {
    mode: 'read',
    doc: doc
  }, ghostEl);
  document.body.innerHTML = ghostEl.innerHTML;
}

function renderInteractiveVersion(doc, mode) {
  document.body.innerHTML = '';
  Component.mount(SheetWriter, {
    mode: mode,
    doc: doc,
    engine: engine,
    onSave: function(doc, changes, cb) {
      var exporter = new SheetHTMLExporter();
      var html = exporter.exportDocument(doc);
      engine.save(html);
    }
  }, document.body);
}

window.Stencila = {};
window.isEditable = true;

function launch() {
  var doc = loadDocument();
  if (window.isEditable) {
    renderStaticReadonlyVersion(doc);
    activateSession(function(err) {
      if (err) throw new Error(err);
      renderInteractiveVersion(doc, 'write');
    });
  } else {
    renderInteractiveVersion(doc, 'read');
  }
}

window.activate = function() {
  engine.activate();
}

window.deactivate = function() {
  engine.deactivate();
}

$(launch);
