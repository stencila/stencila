/**
 * Generate documentation and wrappers for C++ code from the specifications
 * defined in `../meta/*.yaml`
 */

var fs = require('fs'),
    yaml = require('yamljs'),
    nunjucks = require('nunjucks');

nunjucks.configure({
	trimBlocks : true,
	lstripBlocks : true
});

['component', 'stencil', 'sheet'].forEach(function(name){
	var spec = yaml.load('../meta/'+name+'.yaml');
	fs.writeFile(
		'build/' + name + '.hpp',
		nunjucks.render('class.hxx', spec)
	);
	fs.writeFile(
		'build/' + name + '.js',
		nunjucks.render('class.jsx', spec)
	);
});
