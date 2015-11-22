'use strict';

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var $ = require('substance/util/jquery');

var Stencil = require('./model/Stencil');
var StencilWriter = require('./StencilWriter');
var StencilReader = require('./StencilReader');
var StencilViewer = require('./StencilViewer');
var Backend = require("./stencil-backend");

function App() {
  Component.apply(this, arguments);
  this.backend = new Backend();
}

App.Prototype = function() {

  this.getInitialState = function() {
    var doc = new Stencil();
    doc.loadHtml(this.props.html);
    return {
      mode: "write",
      doc: doc
    };
  };

  this.openReader = function() {
    this.extendState({
      mode: 'read'
    });
  };

  this.openPlain = function() {
    this.extendState({
      mode: 'plain'
    });
  };

  this.openWriter = function() {
    this.extendState({
      mode: 'write'
    });
  };

  this.render = function() {
    var el = $$('div').addClass('app');
    var _this = this;

    el.append(
      $$('div').addClass('menu').append(
        $$('button')
          .addClass(this.state.mode ==='write' ? 'active': '')
          .on('click', this.openWriter)
          .append('Write'),
        $$('button')
          .addClass(this.state.mode ==='read' ? 'active': '')
          .on('click', this.openReader)
          .append('Read'),
        $$('button')
          .addClass(this.state.mode ==='plain' ? 'active': '')
          .on('click', this.openPlain)
          .append('Plain')
      )
    );

    if (this.state.doc) {
      var lensEl;
      if (this.state.mode === 'write') {
        lensEl = $$(StencilWriter, {
          doc: this.state.doc,
          onUploadFile: function(file, cb) {
            console.log('custom file upload handler in action...');
            var fileUrl = window.URL.createObjectURL(file);
            cb(null, fileUrl);
          },
          onSave: function(doc, changes, cb) {
            _this.backend.saveDocument(doc, cb);
          },
          onRender: function(doc, cb) {
            _this.backend.renderDocument(doc, cb);
          }
        }).ref('writer');
      } else if (this.state.mode ==='read') {
        lensEl = $$(StencilReader, {
          doc: this.state.doc
        }).ref('reader');
      } else {
        lensEl = $$(StencilViewer, {
          doc: this.state.doc
        }).ref('viewer');
      }
      el.append($$('div').addClass('context').append(lensEl));
    }
    return el;
  };

};

oo.inherit(App, Component);


// Stencila global object is used to indicate that this script
// has loaded successfully and helps dealing with the fallback
// to an alternative host
window.Stencila = {

  // Dynamic loading of scripts
  load : function(source, callback){
    var head = document.getElementsByTagName("head")[0];
    var script = document.createElement("script");
    script.type = "text/javascript";
    script.src  = (window.StencilaHost || '') + source;
    if(callback) script.onload = callback;
    head.appendChild(script);
  }

};

window.Stencila.load('/get/web/ace/ace.js');

window.MathJax = {
  skipStartupTypeset: true,
  showProcessingMessages: false,
  showMathMenu: false,
  "HTML-CSS": {preferredFont: "STIX"}
};
window.Stencila.load('/get/web/mathjax/MathJax.js?config=TeX-MML-AM_HTMLorMML',function(){
  var content = $('#content');
  var html = content.html() || '';
  content.remove();
  Component.mount($$(App,{"html":html}),$('body'));
});
