'use strict';

module.exports = {

  type: 'stencil-figure',
  tagName: 'figure',

  matchElement: function(el) {
    return el.is('figure');
  },

  import: function(el, figure, converter) {
    var $exec = el.find('[data-exec]');
    if($exec.length){
      figure.spec = $exec.attr('data-exec');
      figure.source = $exec.text();
      figure.hash = $exec.attr('data-hash');
      figure.error = $exec.attr('data-error');
    }

    var $img = el.find('[data-out] img');
    if($img.length){
      figure.image = $img.attr('src');
    }

    var $caption = el.find('figcaption,caption');
    if($caption.length){
      figure.caption = converter.annotatedText($caption, [id, 'caption']);
    }
  },

  export: function(figure, el, converter) {
    var $$ = converter.$$;

    if(figure.index) el.attr('data-index',figure.index);

    var exec = $$('<pre>')
      .attr('data-exec',figure.spec)
      .text(figure.source);
    if(figure.hash) exec.attr('data-hash',figure.hash);
    if(figure.error) exec.attr('data-error',figure.error);

    var img = $$('<img>')
      .attr('src',figure.image);

    var out = $$('<div>')
      .attr('data-out','true')
      .append(img);

    var caption = $$('<figcaption>')
      //FIXME: where does id come from?
      .append(converter.annotatedText([id, 'caption']));

    return el.append(exec, out, caption);
  }
};
