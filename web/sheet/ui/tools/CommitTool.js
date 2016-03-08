'use strict';

var Component = require('substance/ui/Component');
var $$ = Component.$$;
var ControllerTool = require('substance/ui/ControllerTool');
var Icon = require('substance/ui/FontAwesomeIcon');

function CommitTool() {
  ControllerTool.apply(this, arguments);
}

CommitTool.Prototype = function() {

  this.render = function() {
    var el = $$('div')
      .addClass('se-tool se-commit-tool')

    if (this.state.disabled) {
      el.addClass('sm-disabled');
    }
    if (this.state.expanded) {
      el.addClass('sm-expanded');
    }

    var button = $$('button')
      .append([
        $$(Icon, {icon: 'fa-compass'}),
        $$('span')
          .addClass('se-label')
          .text('Commit')
      ])
      .on('click', function(){
        this.extendState({
          expanded: !this.state.expanded
        });
        input.focus();
      }.bind(this));
  
    var input = $$('input')
      .attr({
          type: 'text', 
          placeholder: 'Message', 
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
