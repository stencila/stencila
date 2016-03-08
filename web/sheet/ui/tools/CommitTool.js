'use strict';

var extend = require('lodash/object/extend');
var capitalize = require('lodash/string/capitalize');
var Component = require('substance/ui/Component');
var $$ = Component.$$;
var ControllerTool = require('substance/ui/ControllerTool');

function CommitTool() {
  ControllerTool.apply(this, arguments);
}

CommitTool.Prototype = function() {

  this.render = function() {
    var el = $$('div')
      .addClass('se-tool se-commit-tool')
      .attr('title', 'Commit');

    if (this.state.disabled) {
      el.addClass('sm-disabled');
    }
    if (this.state.expanded) {
      el.addClass('sm-expanded');
    }

    var button = $$('button')
      .append(this.props.children)
      .on('click', function(){
        this.extendState({
          expanded: !this.state.expanded
        });
      }.bind(this));
  
    var input = $$('input')
      .attr({
          type: 'text', 
          placeholder: 'Commit message', 
          value: ''
      })
      .htmlProp('autofocus', true)
      .on('change', function(event){
        var message = event.target.value;
        var controller = this.getController()
        controller.getCommand('commit').execute(
          message,
          function(result){
            var logger = controller.getLogger();
            logger.log('New commit created: ' + result.id);
          }
        )
        this.extendState({
          expanded: false
        });
      }.bind(this));

    el.append([button,input]);

    return el;
  };
};

ControllerTool.extend(CommitTool);
CommitTool.static.name = 'commit';
CommitTool.static.command = 'commit';

module.exports = CommitTool;
