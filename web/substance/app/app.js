'use strict';

// var oo = require('substance/util/oo');
// var Component = require('substance/ui/Component');
// var $$ = Component.$$;

var Backend = require("./backend");
var $ = require('substance/util/jquery');
var backend = new Backend();

backend.getDocument('sample', function(err, doc) {
  console.log('my document', doc);
}.bind(this));

// var LensWriter = require('../LensWriter');
// var LensReader = require('../LensReader');

// function App() {
//   Component.Root.apply(this, arguments);
//   this.backend = new Backend();
// }

// App.Prototype = function() {
//   this.openReader = function() {
//     this.extendState({
//       mode: 'read'
//     });
//   };

//   this.openWriter = function() {
//     this.extendState({
//       mode: 'write'
//     });
//   };

//   this.render = function() {
//     var el = $$('div').addClass('app');

//     el.append(
//       $$('div').addClass('menu').append(
//         $$('button')
//           .addClass(this.state.mode ==='write' ? 'active': '')
//           .on('click', this.openWriter)
//           .append('Write'),
//         $$('button')
//           .addClass(this.state.mode ==='read' ? 'active': '')
//           .on('click', this.openReader)
//           .append('Read')
//       )
//     );
    
//     if (this.state.doc) {
//       var lensEl;
//       if (this.state.mode === 'write') {
//         lensEl = $$(LensWriter, {
//           doc: this.state.doc,
//           onUploadFile: function(file, cb) {
//             console.log('custom file upload handler in action...');
//             var fileUrl = window.URL.createObjectURL(file);
//             cb(null, fileUrl);  
//           },
//           onSave: function(doc, changes, cb) {
//             console.log('custom save handler in action...', doc.toXml());
//             cb(null);
//           }
//         }).ref('writer');
//       } else {
//         lensEl = $$(LensReader, {
//           doc: this.state.doc
//         }).ref('reader');
//       }
//       el.append($$('div').addClass('context').append(lensEl));
//     }
//     return el;
//   };

//   this.didMount = function() {
//     this.backend.getDocument('sample', function(err, doc) {
//       this.setState({
//         mode: 'write',
//         doc: doc
//       });
//     }.bind(this));
//   };
// };

// oo.inherit(App, Component);

// $(function() {
//   Component.mount($$(App), $('#container'));
// });
