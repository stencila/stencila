'use strict';
/* global MathJax */

var oo = require('substance/util/oo');
var Component = require('substance/ui/Component');
var $$ = Component.$$;

var $ = window.$ = window.jQuery = require('substance/util/jquery');
require('jquery.hotkeys');

var StencilWriter = require('./StencilWriter');
var StencilReader = require('./StencilReader');
var StencilViewer = require('./StencilViewer');

var StencilRemoteEngine = require('./engine/StencilRemoteEngine');
var engine = new StencilRemoteEngine();

var StencilHTMLImporter = require('./model/StencilHTMLImporter');
var importer = new StencilHTMLImporter();

function App() {
  Component.apply(this, arguments);
  this.engine = engine;
}

App.Prototype = function() {

  this.getInitialState = function() {
    var doc = importer.importDocument(this.props.html);
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
          engine: engine,
          onUploadFile: function(file, cb) {
            console.log('custom file upload handler in action...');
            var fileUrl = window.URL.createObjectURL(file);
            cb(null, fileUrl);
          },
          onSave: function(doc, changes, cb) {
            _this.engine.save(doc, cb);
          },
          onRender: function(doc, cb) {
            _this.engine.render(doc, cb);
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

function AppLaunch(){
  var content = $('#content');
  var html = content.html() || '';
  content.remove();
  Component.mount(App, {"html":html}, $('body'));
  engine.boot();
}

// Stencila global object is used to indicate that this script
// has loaded successfully and helps dealing with the fallback
// to an alternative host
window.Stencila = {

  // Dynamic loading of scripts
  load : function(source, callback) {
    var head = document.getElementsByTagName("head")[0];
    var script = document.createElement("script");
    script.type = "text/javascript";
    script.src  = (window.StencilaHost || '') + source;
    if(callback) script.onload = callback;
    head.appendChild(script);
  }
};

// Load ACE editor
window.Stencila.load('/get/web/ace/ace.js');

// Functions for adding and removing MathJax
function MathJaxAdd(){
  // Initial render of MathJax
  // Render using 'Rerender' instead of 'Typeset'
  // because math is already in <script type="math/..."> elements
  MathJax.Hub.Queue(
    ["Rerender",MathJax.Hub,"content"],
    function() {
      // Hide math script elements which should now have been rendered into
      // separate display elements by MathJax
      $('#content').find('script[type^="math/tex"],script[type^="math/asciimath"]').each(function() {
        $(this).css('display','none');
      });
    }
  );
}
function MathJaxRemove() {
  var $content = $('#content');
  // Get all MathJax "jax" elements (e.g.
  //    <script type="math/asciimath" id="MathJax-Element-2">e=m^2</script>
  // ) and remove the id if it starts with MathJax
  $content.find('script[type^="math/asciimath"],script[type^="math/tex"]').each(function(){
    var elem = $(this);
    if(/^MathJax/.exec(elem.attr('id'))) elem.removeAttr('id');
    // Remove the css style added above to hide these
    elem.removeAttr('style');
  });
  // Remove all elements which have been added
  $content.find('.MathJax_Error, .MathJax_Preview, .MathJax').remove();
}

// Configure and load MathJax
window.MathJax = {
  skipStartupTypeset: true,
  showProcessingMessages: false,
  showMathMenu: false,
  "HTML-CSS": {preferredFont: "STIX"}
};

window.Stencila.load('/get/web/mathjax/MathJax.js?config=TeX-MML-AM_HTMLorMML', function() {
  $(function() {
    if(window.location.host.match('stenci.la|localhost')){
      if(window.location.host.match('localhost')){
        console.info('If necessary, use 127.0.0.1:'+window.location.port+' to prevent Stencil UI loading');
      }
      AppLaunch();
    } else {
      // At present, some stencils hosted elsewhere (e.g. github.io and file://)
      // may not have content (e.g. sections, tables) that can yet be rendered by the full editor/viewer UI
      console.info('Not loading Stencil UI. Use stenci.la or localhost:'+window.location.port+' if you would like it.');
      // Override some of the integration styles and do MathJax rendering only
      $('html,body').css({
        overflow: 'auto'
      });
      MathJaxAdd();
    }
  });
});
