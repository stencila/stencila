'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var $ = window.$ = require('substance/util/jquery');

// var SheetEditor = require('./ui/SheetEditor');
var SheetWriter = require('./ui/SheetWriter');

var SheetHTMLImporter = require('./model/SheetHTMLImporter');

function App() {
	Component.apply(this, arguments);
}

App.Prototype = function() {

  this.getInitialState = function() {
    return { mode: "initial" };
  };

  this.render = function() {
    var el = $$('div').addClass('app');
    if (this.state.mode === "initial") {
      el.html(this.props.html);
    } else {
      el.append($$(SheetWriter, {
        doc: this.state.doc
      }));
    }
    return el;
  };

  this.didMount = function() {
    if (!this.state.doc) {
      // we are doing this, so that we do not run into problems
      // such as infinite loops during window.onload()
      setTimeout(function() {
        var importer = new SheetHTMLImporter();
        var doc = importer.importDocument(this.props.html);
        console.log('Imported sheet...');
        this.extendState({
          mode: 'writer',
          doc: doc
        });
      }.bind(this));
    }
  };

};

oo.inherit(App, Component);

window.Stencila = {};

function launch() {
  var content = $('#content');
  var html = content.html() || '';
  content.remove();
  Component.mount(App, {"html":html}, document.body);
}

$(launch);
