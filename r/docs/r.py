'''
Module for generating Stencila R documentation.
Converts Rd files in the stencila R package to html.

Copyright (c) 2012-2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
'''
import os
import shutil
import distutils.dir_util
import subprocess
import re

from bs4 import BeautifulSoup

import pygments
import pygments.lexers
import pygments.formatters

rds_dir = "../stencila/man/"
template = None
pre_lexer = None
pre_formatter = None

def startup():
    # Read in template
    global template
    template = file('template.html').read()
    # Copy in R specific CSS
    shutil.copyfile("r.css","html/r.css")
    # Set up syntax highlighting
    global pre_lexer, pre_formatter
    pre_lexer = pygments.lexers.SLexer()
    pre_formatter = pygments.formatters.HtmlFormatter(linenos=False, cssclass="code")
    file("html/code.css","w").write(pre_formatter.get_style_defs())

def version():
    return subprocess.Popen(
        'git describe', 
        stdout=subprocess.PIPE,
        shell=True
    ).communicate()[0]

def rds():
    '''
    Get a list of Rd files in the R packages
    '''
    return os.listdir(rds_dir)
    
def convert(rd):
    return subprocess.Popen(
        '''R CMD Rdconv --type=html %s%s'''%(rds_dir,rd),
        stdout=subprocess.PIPE,
        shell=True
    ).communicate()[0]

def combine():
    all = ''
    for rd in rds():
        print rd
        html = convert(rd)
        soup = BeautifulSoup(html)
        # Extract the name of the Rd from the first table and then delete the table
        name = soup.body.table.tr.td.string
        soup.body.table.decompose()
        # Replace <body> with a <div>
        soup.body.wrap(soup.new_tag('div',**{'class':'rd'}))
        soup.div.body.unwrap()
        # Add name to title h2
        soup.div.h2.insert(0,name+":")
        # Find any <pre> elements and highlight them
        for pre in soup.find_all('pre'):
            div = BeautifulSoup(pygments.highlight(pre.string, pre_lexer, pre_formatter))
            pre.replace_with(div)
        # Append to output by converting to string
        # Do not use .prettify() becausee that makes up formatting in code blocks
        all += str(soup.div)
    return all

def generate():
    print>>file('html/index.html','w'), template%{
        'version' : version(),
        'content' : combine()
    }
    
if __name__=='__main__':
    startup()
    generate()
