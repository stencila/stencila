// A snippet of code which could be a based class for all inline node tools:
// - creates using commend if necessay
// - deletes if appropriate - and puts back to text
// -

// Create a node if necessary, or toggle the language, or change to plain text
/*
if (!node) {

  if (!this.props.disabled) this.performAction();

} else {

  session.transaction(function (tx) {

    tx.delete(node.id);
    var result = insertText(tx, {
      selection: session.getSelection(),
      text: node.source
    });

  });

}
*/
