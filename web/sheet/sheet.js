'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var $ = window.$ = require('substance/util/jquery');

var Sheet = require('./model/Sheet');
var SheetComponent = require('./ui/SheetComponent');
var SheetHTMLImporter = require('./model/SheetHTMLImporter');

function App() {
	Component.apply(this, arguments);
}

App.Prototype = function() {

  this.getInitialState = function() {
    var importer = new SheetHTMLImporter();
    var doc = importer.importDocument(this.props.html);
    return {
      mode: "write",
      doc: doc
    };
  };

  this.render = function() {
    var el = $$('div').addClass('app');
    el.append($$(SheetComponent, {
      doc: this.state.doc
    }));
    return el;
  };

};

oo.inherit(App, Component);

function launch() {
  var content = $('#content');
  var html = content.html() || '';
  content.remove();
  Component.mount($$(App, {"html":html}), $('body'));
}

$(launch);
