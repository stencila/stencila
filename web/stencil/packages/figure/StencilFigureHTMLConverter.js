'use strict';

module.exports = {

  type: 'stencil-figure',
  tagName: 'figure',

  matchElement: function(el) {
    return el.is('figure');
  },

  import: function(el, node, converter) {
    var exec = el.find('[data-exec]');
    if(exec){
      node.spec = exec.attr('data-exec');
      node.source = exec.text();
      node.hash = exec.attr('data-hash');
      node.error = exec.attr('data-error');
    }

    var img = el.find('[data-out] img');
    if(img){
      node.image = img.attr('src');
    }

    var caption = el.find('figcaption,caption');
    if(caption){
      node.caption = converter.annotatedText(caption, [node.id, 'caption']);
    }
  },

  export: function(node, el, converter) {
    var $$ = converter.$$;

    if(node.index) el.attr('data-index', node.index);

    var exec = $$('pre')
      .attr('data-exec', node.spec)
      .text(node.source);
    if(node.hash) exec.attr('data-hash', node.hash);
    if(node.error) exec.attr('data-error', node.error);

    var img = $$('img')
      .attr('src', node.image);

    var out = $$('div')
      .attr('data-out','true')
      .append(img);

    var caption = $$('figcaption')
      //FIXME: where does id come from?
      .append(converter.annotatedText([node.id, 'caption']));

    el.append(exec, out, caption);
  }
};
