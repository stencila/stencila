#!/usr/bin/env Rscript

# Stencila command line interface (CLI) for R
# 
# Allows you to launch an R session, grab a component and execute methods from
# the terminal shell.

# Check if outputting to a terminal or not
terminal <- isatty(stdout())

# ANSI escape codes for terminal text colours.
# See http://en.wikipedia.org/wiki/ANSI_escape_code
if(terminal){
    grey <- '\x1b[90m'
    green <- '\x1b[32m'
    yellow <- '\x1b[93m'
    blue <- '\x1b[94m'
    magenta <- '\x1b[95m'
    cyan <- '\x1b[96m'
    reset <- '\x1b[0m'
} else {
    grey <- NULL
    green <- NULL
    yellow <- NULL
    blue <- NULL
    magenta <- NULL
    cyan <- NULL
    reset <- NULL
}

# Output function
# May be overridden below if -q option set
out <- function(...) {
    cat(...)
}
# Error function
error <- function(...) {
    cat(green)
    cat(...)
    quit('no')
}

# Load package
library(stencila,quietly=T)

# Get the command line arguments
commands <- commandArgs(trailingOnly=TRUE)

# Set options
options <- list(
    help=FALSE
)
if(length(commands)>0 && substr(commands[1],1,1)=='-') {
    # First command is actually list of options
    # So pop them off the commands list
    options_string = substr(commands[1],1,nchar(commands[1]))
    commands <- commands[-1]
    for(index in 1:nchar(options_string)){
        option <- substr(options_string,index,index)
        if(option=='q'){
            out <- function(...){}
        } else if (option=='h') {
            options$help <- TRUE
        }
    }
}
# If no real commands then must provide help
if(length(commands)==0) {
    options$help <- TRUE
}

# Banner
out(green,'Stencila ',stencila:::version(),' CLI for R\n\n',reset,sep='')

# General help on usage
if(options$help){
    out(green)
    out('Usage:\n')
    out('  stencila-r -[options]                             \n')
    out('  stencila-r <type> <method>[:<arg>,<arg>,...]  \n')
    out('  stencila-r <address> <method>[:<arg>,<arg>,...] ...\n')
    out('\n')
    out('Examples:\n')
    out('  stencila-r stencil view ...\n')
    out('  stencila-r . update write\n')
    quit('no')
}

# Get target
target <- commands[1]
if(target=="-"){
    component <- NULL
} else if(target %in% c('stencil','sheet')){
    out('Creating new       : ',magenta,target,reset,'\n',sep='')
    if(target=='stencil'){
        component <- Stencil()
    }
    else if(target=='sheet'){
        component <- Sheet()
    }
    else {
        error('Unknown component type:',target)
    }
    component$path('') # Ensure component is given a path
} else {
    out('Grabbing from      : ',magenta,target,reset,'\n',sep='')
    component <- grab(target)
}

if(!is.null(component)){
    # Confirm component address, path, type
    out('Component address  : ',cyan,component$address(),reset,'\n',sep='')
    out('Component path     : ',cyan,component$path(),reset,'\n',sep='')
    out('Component type     : ',cyan,class(component)[1],reset,'\n',sep='')
}

# Iterate over methods, applying them to the component
commands <- commands[-1]
for(command in commands){
    if(!is.na(command)){
        # If elipsis then sleep until Ctrl+C
        if(command=='...'){
            out('\nSleeping (use Ctrl+C to exit)\n')
            Sys.sleep(Inf)
        }
        # Get method and arguments
        parts <- strsplit(command,':')[[1]]
        method <- parts[1]
        args <- parts[2]
        # Parse args into numbers if possible, otherwise assume
        # they are strings
        args_formatted <- NULL
        if(!is.na(args)){
            for(arg in strsplit(args,',')[[1]]){
                number <- suppressWarnings(as.numeric(arg))
                if(is.na(number)) args_formatted <- c(args_formatted,paste0('"',arg,'"'))
                else args_formatted <- c(args_formatted,arg)
            }
            args_formatted <- paste0(args_formatted,collapse=',')
        }
        # Construct call
        if(!is.null(args_formatted)) call <- paste0(method,'(',args_formatted,')')
        else call <- paste0(method,'()')
        # Execute call
        out('Running method     : ',blue,call,reset,'\n',sep='')
        out(yellow)
        if(!is.null(component)) call <- paste0('component$',call)
        result <- eval(parse(text = call))
        # Only show if not null return and not the component
        if(!is.null(result) && !inherits(result,class(component)[1])) show(result)
        out(reset)
    }
}

