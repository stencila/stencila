'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var $ = window.$ = require('substance/util/jquery');

var SheetComponent = require('./ui/SheetComponent');
var SheetHTMLImporter = require('./model/SheetHTMLImporter');

function App() {
	Component.apply(this, arguments);
}

App.Prototype = function() {

  this.getInitialState = function() {
    return { mode: "loading" };
  };

  this.render = function() {
    var el = $$('div').addClass('app');
    if (this.state.mode === "loading") {
      el.text('Loading...');
    } else {
      el.append($$(SheetComponent, {
        doc: this.state.doc
      }));
    }
    console.log('Rendered App...');
    return el;
  };

  this.didMount = function() {
    if (!this.state.doc) {
      // we are doing this, so that we do not run into problems
      // such as infinite loops during window.onload()
      setTimeout(function() {
        var importer = new SheetHTMLImporter();
        console.log('Importing HTML...');
        var doc = importer.importDocument(this.props.html);
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
  Component.mount($$(App, {"html":html}), $('body'));
}

$(launch);
