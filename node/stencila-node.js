#!/usr/bin/env node

var stencila = require('stencila-node'),
    colors = require('colors'),
    readline = require('readline');

// Banner
console.log('Stencila CLI for Node.js'.green);

// Get commands (remove first two arguments)
var commands = process.argv.slice(2);

// Help
if (commands.length == 0 || (
  commands.length == 1 && commands[0] == 'help'
)){
    console.log(
"\nUsage:\n\
  stencila-node help \n\
  stencila-node <address> <method>[:<arg>,<arg>,...] ...\n\
  stencila-node stencil|sheet <method>[:<arg>,<arg>,...] ...\n\
\n\
Examples:\n\
  stencila-node . render export:index.html\n\
  stencila-node sheet view ...\n"
    );
    process.exit();
}

// Target of commands
var target = commands.shift();
if (target == 'stencil' || target == 'sheet') {
  console.log('Creating new       : ', target.red);
  var clas = target.charAt(0).toUpperCase() + target.substr(1).toLowerCase();
  component = stencila[clas]();
  component.path('');
} else {
  console.log('Grabbing from      : ', target.red);
  // TODO
}

// Confirm component address, path, type
console.log('Component address  : ', component.address().cyan);
console.log('Component path     : ', component.path().cyan);

// Iterate over commands
commands.forEach(function(command){
  if (command == '...') {
    var line = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });
    line.question('Waiting, press any key to continue > ', function() {
      line.close();
    });
  } else {
    var method = command;
    console.log('Running method     : ', method.blue);
    component[method]();
  }
});
