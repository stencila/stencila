#R"(

import inspect
import code
import sys

PY = sys.version_info[0]

if PY==2:
	from cStringIO import StringIO
elif PY==3:
	from io import StringIO

# Create an `exec_` function that can be used in both
# Python 2 and 3. This is inspired by the exec_ function implmented in
# https://pypi.python.org/pypi/six but simplified for this use case
if PY==2:
	# Function signature compatible with Python 3's
	def exec_(code, globals, locals):
		assert globals is None
		exec("""exec code in {}, locals""")
elif PY==3:
	import builtins
	# Need to use `getattr` to avoid Python 2 syntax error
	exec_ = getattr(builtins,'exec')

# Creat a `next` function as in Python 3
if PY==2:
	def next(iterator): return iterator.next()

class Images:
	'''
	Handles image generation
	'''

	engine = None
	count = 0

	@staticmethod
	def filename(format):
		Images.count += 1
		return "%d.%s"%(Images.count,format)

# Attempt to load image packages
try:
	import matplotlib
except ImportError:
	pass
else:
	Images.engine = 'matplotlib'
	# Use the cairo backend because it support a variety of image formats.
	matplotlib.use('cairo')
	# Only once backend has been set then pylab can be imported
	import pylab


class Namespace(dict):

	def __init__(self,parent=None):
		self.parent = parent

	def __getitem__(self,name):
		try:
			return dict.__getitem__(self, name)
		except KeyError:
			if self.parent: return self.parent[name]
			else: raise KeyError(name)


class Console(code.InteractiveConsole):
	# Used for `interact` below
	# See https://docs.python.org/2/library/code.html

	def __init__(self,locals):
		code.InteractiveConsole.__init__(self,locals)

	def push(self,source):
		# Push some source to the buffer
		# Redirect sys.stdout because that is what gets used for
		# both printing (i.e. via "print") and the "showing" of a value
		stdout = sys.stdout
		sys.stdout = StringIO()
		# Reset error string
		self.err = ""
		# Run code
		more = code.InteractiveConsole.push(self,source)
		# Get output and restore sys.stdout
		self.out = sys.stdout.getvalue()
		sys.stdout = stdout
		return more

	def write(self,line):
		# Capture a line of traceback (lines have a newline at end)
		self.err += line

	def interact(self,source):
		# Push some source and then concatenate output and errors
		# Note that currently di not use return value of push()
		self.push(source)
		return self.out+self.err


class Context:

	def __init__(self,namespace=None):
		if namespace is None: namespace = Namespace()
		self.namespaces = [namespace]

	# Bind this Python side object to the C++ side
	
	def bind(self,callback):
		self.set("__callback__",callback)

	# Shortcut methods for accessing the namespace stack
	
	def push(self,namespace=None):
		# Push a new namespace on to the stack
		if namespace is None: namespace = Namespace(self.top())
		self.namespaces.append(namespace)
		return self

	def pop(self):
		# Pop the current namespace off the top of the stack
		self.namespaces.pop(len(self.namespaces)-1)
		return self

	def top(self):
		# Get the top of the namepace stack (i.e. the current namespace)
		return self.namespaces[len(self.namespaces)-1]

	def get(self,name):
		# Get a variable from the top of the stack
		return self.top()[name]

	def set(self,name,value):
		# Set a variable in the top of the stack
		self.top()[name] = value
		return self

	def evaluate(self,expression):
		# Evaluate an expression in the top of the stack
		return eval(expression,{},self.top())

	# Context methods that provide the interface defined in cpp/stencila/context.hpp

	def execute(self,code,format=None,width=None,height=None,units=None):
		# Some Python versions don't like trailing blank lines so remove all surrounding
		# whitepace
		code = code.strip()
		# Execute in top namespace
		exec_(code,None,self.top())
		# Return according to format code
		if format in ('png','svg'):
			if Images.engine=='matplotlib':
				filename = Images.filename(format)
				pylab.savefig(filename)				
				return filename

		return ""

	def interact(self,code):
		# Note that there is no buffering done here
		# since a new console in instantiated each time
		return Console(self.top()).interact(code+"\n")

	def assign(self,name,expression):
		self.top()[name] = self.evaluate(expression)

	def write(self,expression):
		return str(self.evaluate(expression))

	def test(self,expression):
		return bool(self.evaluate(expression))

	def mark(self,expression):
		self.enter()
		self.set('__subject__',self.evaluate(expression))

	def match(self,expression):
		try: subject = self.get('__subject__')
		except KeyError: raise ContextError('No subject defined for switch directive')
		return subject==self.evaluate(expression)

	def unmark(self):
		self.exit()

	def begin(self,item,expression):
		self.enter()
		self.set('__item__',item)
		items = self.evaluate(expression)
		iterator = items.__iter__()
		self.set('__items__',iterator)
		return self.next()

	def next(self):
		try:
			item = next(self.get('__items__'))
		except StopIteration:
			return False
		else:
			self.set(self.get('__item__'),item)
			return True

	def enter(self,expression=""):
		if len(expression)>0:
			self.push(self.evaluate(expression))
		else:
			self.push()

	def exit(self):
		self.pop()

class ContextError(BaseException):
	pass 

#)"
