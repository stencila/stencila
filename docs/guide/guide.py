'''
A Python module for generating the Stencila users guide. This module
takes HTML files in the "source" directory and processes them to:
    - add some standard style, navigation and other elements
    - generate a table of contents from h1, h2 etc elements
    - add syntax highlighting to code blocks
    - compile and run code blocks to ensure that they work!
    - insert output from code blocks
Not all of these functions are currently implemented.

Within source HTML, <script> elements are used for code blocks because all characters,
including "<",">" etc can be used within them.
These <script> elements have a class attribute which is used to indicate the language.
This is use for both syntax highlighting and for compiling/running. Valid language codes are:
    cpp: C++
    py: Python
    r: R

Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
'''
import HTMLParser
import shutil
import os
import re
import distutils.dir_util

from pygments import highlight
from pygments.lexers import get_lexer_by_name
from pygments.formatters import HtmlFormatter

class Processor(HTMLParser.HTMLParser):
    
    def __init__(self,name):
        HTMLParser.HTMLParser.__init__(self)
        
        self.name = name
        self.counter = 0
        self.language = None
        
        self.code = {
            'cpp': file('cpp/%s.cpp'%self.name,'w'),
            'py': file('py/%s.py'%self.name,'w'),
            'r': file('r/%s.r'%self.name,'w'),
        }
        
        self.html = ''
        
    def handle_starttag(self, tag, attrs):
        #Convert attr tuple to a dict
        attrs = dict(attrs)
        if tag=='div' and attrs.get('class')=='code':
            self.counter += 1
        if tag=='script':
            # Extract the language code from the tag attributes
            language = attrs['class']
            if language not in ('cpp','py','r'): raise Exception('Could not extract a valid language code from tag attributes:'+str(attrs))
            self.language = language
            # Replace the scipt tag with a div tag
            tag = 'div'
        self.html += '<%s %s>\n'%(tag,' '.join(['%s="%s"'%(name,value) for name,value in attrs.items()]))
        
    def handle_endtag(self, tag):
        if self.language:
            self.language = None
            tag = 'div'
        self.html += '</%s>\n'%tag
        
    def handle_data(self, data):
        if self.language:
            # If in a <script> element then...
            # Get the code file...
            code = self.code[self.language]
            # Create a redict for standard output
            output = '%s.%i.out'%(self.name,self.counter)
            print>>code,{
                'cpp' : 'if(!std::freopen("%s","w",stdout)) throw Exception("Unable to redirect output to %s");'%(output,output),
                'py' : 'sys.stdout = file("%s","w")'%output,
                'r' : 'sink("%s")'%output,
            }[self.language]
            # Write the code snippet
            print>>code,data
            # Format the code snippet
            lexer = get_lexer_by_name(self.language, stripall=True)
            formatter = HtmlFormatter(linenos=False, cssclass="source")
            data = highlight(data, lexer, formatter)
            # Create a place holder for output. The trailing space after the filename is important!
            data += '''<div class="output">%s/%s </div>'''%(self.language,output)
        self.html += data
        
    def cpp_start(self):
        return '''
            #include <iostream>
            #include "../../../cpp/stencila.hpp"

            int main(void){
            using namespace Stencila;
            try {
        '''

    def cpp_finish(self):
        return '''
            } catch (Exception& e) {
                std::cout<<"Exception : "<<e.what();
            }
            catch (...) {
                std::cout<<"Unknown exception";
            }
            return 0;
            }
        '''
        
    def cpp_run(self,opts):
        return '''
            rm -f %(name)s.exe %(name)s.*.out;
            g++ -std=c++0x -Wall -O2 -o %(name)s.exe %(name)s.cpp -lboost_system -lboost_filesystem -lsqlite3;
            ./%(name)s.exe;
        '''%opts

    def py_start(self):
        return '''import sys'''

    def py_finish(self):
        return ''''''

    def py_run(self,opts):
        return '''
            rm -f %(name)s.*.out;
            python %(name)s.py;
        '''%opts

    def r_start(self):
        return ''''''

    def r_finish(self):
        return ''''''

    def r_run(self,opts):
        return '''
            rm -f %(name)s.*.out;
            R --interactive --no-save < %(name)s.r;
        '''%opts
        
    @staticmethod
    def init(pages):
        #Copy files from docs/style
        distutils.dir_util.copy_tree("../style","html/")
        #Copy in guide CSS
        shutil.copyfile("guide.css","html/guide.css")
        #Create pygments CSS
        file("html/code.css","w").write(HtmlFormatter().get_style_defs('.code'))
        #Create a list of links to pages
        Processor.links = ''
        for page in pages:
            Processor.links += '''<li><a href="%s.html">%s</a></li>'''%(page,page.title())
        
    def process(self):
        
        #Read in source file
        source = file('source/'+self.name+".html").read()
        
        #Start code files
        for lang in self.code.keys():
            start = getattr(self,lang+'_start')()
            print>>self.code[lang],start

        #Parse source file
        self.feed(source)
        
        #Finish code files
        for lang in self.code.keys():
            finish = getattr(self,lang+'_finish')()
            print>>self.code[lang],finish
            self.code[lang].close()
            
        #Copy data to language directory, clean and run code
        for lang in self.code.keys():
            distutils.dir_util.copy_tree("data",lang)
            os.chdir(lang)
            os.system(getattr(self,lang+'_run')({
                'name':self.name
            }))
            os.chdir("..")
        
        #Parse HTML and replace output with the generated output
        matches = re.compile('<div class="output">(.*)\s</div>').finditer(self.html)
        for match in matches:
            filename = match.group(1)
            
            try:
                output = file(filename).read().strip()
            except IOError:
                output = '<span class="splat">Arrrgh, splat!</span>'
                
            if len(output)>0:
                output = output.replace("\n",'<br>')
                self.html = self.html.replace(filename,output)
            else:
                self.html = self.html.replace('<div class="output">%s </div>'%filename,'')
        
        #Obtain repository version number
        from subprocess import Popen, PIPE
        p = Popen('git describe', shell=True,stdout=PIPE, stderr=PIPE)
        version, stderr = p.communicate()
        
        #Wrap html
        html = '''<!DOCTYPE html>
<html>

    <head>
        <link rel="stylesheet" href="guide.css">
        <link rel="stylesheet" href="guide.js">
    </head>
    
    <body>
    
    <div class="header">
        <a href="http://stenci.la"><img alt="Stencila" src="stencila.png"></a>
	<div class="details">
		<h1>User guide</h1>
		<h2>Version %(version)s</h2>
	</div>
        <ul class="links">
            %(links)s
        </ul>
    </div>
    
    <div class="page">
        %(page)s
    </div
    
    </body>
    
</html>
'''%{
            'version': version,
            'links' : self.links,
            'page': self.html
        }
        
        #Print to output file
        print>>file("html/%s.html"%self.name,"w"), html

if __name__=='__main__':
          
    pages = ('datatable','dataset')
    
    Processor.init(pages)
    for page in pages:
        p = Processor(page)
        p.process()


