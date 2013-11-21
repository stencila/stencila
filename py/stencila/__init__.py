import inspect
import sys

from extension import *

Null = Datatype('n')
Integer = Datatype('i')
Real = Datatype('r')
Text = Datatype('t')

class Table(TableBase):
    
    def enter(self,names=None,frame=None):
        '''
        Import the Datatable's Columns into the calling
        frame's local namespace.

        This is a bit of a hack and probably should not be used in
        production code without some testing.
        Does not seem to work under unittest, perhaps
        because both use trace functions.

        This method uses the hack described at http://code.google.com/p/ouspg/wiki/AnonymousBlocksInPython
        where a trace function is used to update a frame's locals.
        An alternative might be byte code hacking as described
        at http://www.voidspace.org.uk/python/articles/code_blocks.shtml.
        '''
        if names is None: names = self.names()
        if frame is None: frame = inspect.currentframe(1)
        
        columns = dict((name,Column(name)) for name in names)
            
        def trace(frame, event, arg):
            frame.f_locals.update(columns)
            del frame.f_trace

        sys.settrace(lambda *args, **kwargs: None)
        frame.f_trace = trace
            
    def exit(self,names=None,frame=None):
        if names is None: names = self.names()
        if frame is None: frame = inspect.currentframe(1)
        for name in names:
            del frame.f_locals[name]
        
    def __enter__(self):
        self.enter(frame=inspect.currentframe(1))
        
    def __exit__(self, type, value, traceback):
        self.exit(frame=inspect.currentframe(1))
        
    def __getitem__(self,key):
        # The key arg is either a single object or a tuple of objects. 
        # So, convert to a tuple for consistency
        if not isinstance(key,tuple): key = (key,)
        print key

    

