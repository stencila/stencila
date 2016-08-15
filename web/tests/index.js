var glob = require('glob');
glob.sync(__dirname + '/**/*.test.js').forEach(function(path){
	require(path);
});
