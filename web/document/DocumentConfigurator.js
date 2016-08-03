'use strict';

var Configurator = require('substance/util/Configurator');

/**
 * A "configurator" for a document.
 * 
 * Uses the Substance package mechanism to reduce repetition.
 * See `substance/util/AbstractConfigurator` for inherited methods
 * used by `DocumentHTMLImporter`, `DocumentEditor` etc
 *
 * @class      DocumentConfigurator (name)
 */
function DocumentConfigurator() {
  DocumentConfigurator.super.apply(this, arguments);

  // Define the schema (used by importer)
  this.defineSchema({
    name: 'stencila-document',
    defaultTextType: 'paragraph'
  });

  // Import Substance packages, in alphabetical order
  this.import(require('substance/packages/base/BasePackage'));
  this.import(require('substance/packages/blockquote/BlockquotePackage'));
  this.import(require('substance/packages/code/CodePackage'));
  this.import(require('substance/packages/emphasis/EmphasisPackage'));
  this.import(require('substance/packages/heading/HeadingPackage'));
  this.import(require('substance/packages/image/ImagePackage'));
  this.import(require('substance/packages/link/LinkPackage'));
  this.import(require('substance/packages/list/ListPackage'),{
    enableMacro: true
  });
  this.import(require('substance/packages/paragraph/ParagraphPackage'));
  this.import(require('substance/packages/strong/StrongPackage'));
  this.import(require('substance/packages/subscript/SubscriptPackage'));
  this.import(require('substance/packages/superscript/SuperscriptPackage'));
  this.import(require('substance/packages/table/TablePackage'));

  // "Overrides" of Substance nodes
  // Link (overrides of command and tool)
  this.addCommand('link', require('./nodes/link/LinkCommand'), {nodeType: 'link'});
  this.addTool('link', require('./nodes/link/LinkTool'));
  // Codeblock (deals with language in import/export and component)
  this.import(require('./nodes/codeblock/CodeblockPackage'));

  // Icons, not defined in Substance packages but used in our `BlockToolset`
  this.addIcon('heading', { 'fontawesome': 'fa-header' });
  this.addIcon('paragraph', { 'fontawesome': 'fa-paragraph' });
  this.addIcon('list', { 'fontawesome': 'fa-list' });
  this.addIcon('table', { 'fontawesome': 'fa-table' });
  this.addIcon('image', { 'fontawesome': 'fa-image' });
  this.addIcon('blockquote', { 'fontawesome': 'fa-quote-right' });
  this.addIcon('codeblock', { 'fontawesome': 'fa-code' });


  // Import Stencila node packages
  this.import(require('./nodes/math/MathPackage'));

  this.import(require('./nodes/execute/ExecutePackage'));
  this.import(require('./nodes/print/PrintPackage'));

  this.import(require('./nodes/mark/MarkPackage'));
  this.import(require('./nodes/discussion/DiscussionPackage'));
  this.import(require('./nodes/comment/CommentPackage'));

  // Import editor packages
  this.import(require('./editors/visual/VisualEditorPackage'));

}

DocumentConfigurator.Prototype = function() {

  /**
   * Gets the file client
   * 
   * Method required by `AbstractEditor._initialize`
   *
   * @return     {<type>}  The file client.
   */
  this.getFileClient = function() {
    return null;
  };

  /**
   * Gets the save handler.
   *
   * Method required by `AbstractEditor._initialize`
   * 
   * @return     {<type>}  The save handler.
   */
  this.getSaveHandler = function() {
    return null;
  };

};

Configurator.extend(DocumentConfigurator);

module.exports = DocumentConfigurator;
