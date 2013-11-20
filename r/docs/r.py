'''
Module for generating Stencila R documentation.
Converts Rd files in the stencila R package to html.
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

sections = [
    ['Dataset',[
        'Dataset',
        'Dataset-uri','Dataset-tables','Dataset-indices',
    ]],
    ['Datatable',[
        'Datatable',
        'Datatable-subscript',
        'Datatable-head','Datatable-tail','Datatable-dim',
        'Datatable-as.data.frame'
    ]],
    ['Datacursor',[
        'Datacursor'
    ]],
    ['Dataquery',[
        'Dataquery'
    ]],
    ['Stencil',[
        'Stencil',
        'Stencil-load','Stencil-render'
    ]],
    ['Utility',[
        'version', 'iterate'
    ]],
    ['Other',[]] #Gets filled in by startup()
]

# A list of Rds that should NOT be included
exclude = ['stencila','Column','Constant']

def startup():
    # Read in template
    global template
    template = file('template.html').read()
    # Copy in R specific CSS
    shutil.copyfile("r.css","html/r.css")
    # Set up syntax highlighting
    global pre_lexer, pre_formatter
    pre_lexer = pygments.lexers.SLexer()
    pre_formatter = pygments.formatters.HtmlFormatter(style="native",linenos=False, cssclass="code")
    file("html/code.css","w").write(pre_formatter.get_style_defs())
    # Add additional Rds to the sections list
    #Get a list of Rd files in the R packages
    included = []
    for section,rds in sections: included += rds
    rds = os.listdir(rds_dir)
    rds = [rd[:-3] for rd in rds]
    for rd in rds:
        if not rd in exclude and not rd in included:
            sections[len(sections)-1][1].append(rd)

def version():
    '''
    Get the version of Stencila from the Makefile
    '''
    return subprocess.Popen(
        'make --no-print-directory --quiet version', 
        stdout=subprocess.PIPE,
        shell=True
    ).communicate()[0].strip()

def convert(rd):
    return subprocess.Popen(
        '''R CMD Rdconv --type=html %s%s'''%(rds_dir,rd),
        stdout=subprocess.PIPE,
        shell=True
    ).communicate()[0]

def combine():
    all = ''
    for name,rds in sections:
        print name
        all += "<section><h1>%s</h1>"%name
        for rd in rds:
            print rd
            html = convert(rd+".Rd")
            soup = BeautifulSoup(html)
            # Extract the name of the Rd from the first table and then delete the table
            name = soup.body.table.tr.td.string
            soup.body.table.decompose()
            # Extract the brief description and then delete it
            brief = soup.body.h2.string
            soup.body.h2.decompose()
            # Replace <body> with a <div>
            soup.body.wrap(soup.new_tag('div',**{'class':'details'}))
            soup.div.body.unwrap()
            # Find any <pre> elements and highlight them
            for pre in soup.find_all('pre'):
                div = BeautifulSoup(pygments.highlight(pre.string, pre_lexer, pre_formatter))
                pre.replace_with(div)
            # Append to output by converting to string
            # Do not use .prettify() because that mucks up formatting in code blocks
            all += '''<div class="rd">
                <div class="brief"><h1>%s : %s</h1></div>
                %s
            </div>'''%(name,brief,str(soup.div))
        all += "</section>"
    return all

def generate():
    print>>file('html/index.html','w'), template%{
        'version' : version(),
        'content' : combine()
    }
    
if __name__=='__main__':
    startup()
    generate()
