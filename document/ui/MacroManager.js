'use strict';

import MacroManagerBase from 'substance/ui/MacroManager'

/**
 * A custom `MacroManager` which excludes text in existing annotations
 * from macros regex test strings.
 *
 * @class      MacroManager (name)
 */
function MacroManager () {
  MacroManager.super.apply(this, arguments);
}

MacroManager.Prototype = function () {
  // This function is from `substance/ui/MacroManager` except for the part
  // labelled as "Modification" below
  this.executeMacros = function (update, info) {
    var change = update.change;
    if (!change) {
      return;
    }

    var doc = this.context.documentSession.getDocument();
    var nodeId, node, text;
    var path;
    if (info.action === 'type') {
      // HACK: we know that there is only one op when we type something
      var op = change.ops[0];
      path = op.path;
      nodeId = path[0];
      node = doc.get(nodeId);
      text = doc.get(path);
    } else {
      return;
    }

    // Modification: converts text within existing annotations to spaces so
    // that they are not matched in subsequent macros
    var annos = doc.getIndex('annotations').get(path);
    annos.forEach(function (anno) {
      var length = anno.endOffset - anno.startOffset + 1;
      text = text.substring(0, anno.startOffset) + Array(length + 1).join(' ') + text.substring(anno.endOffset + 1);
    });

    var props = {
      action: info.action,
      node: node,
      path: path,
      text: text,
      selection: this.context.documentSession.getSelection()
    };
    for (var i = 0; i < this.macros.length; i++) {
      var macro = this.macros[i];
      var executed = macro.execute(props, this.context);

      if (executed) {
        break;
      }
    }
  };
};

MacroManagerBase.extend(MacroManager);

module.exports = MacroManager;
