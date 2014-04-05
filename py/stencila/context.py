from namespace import *

class ContextError(Exception):
	'''
	An exception caused by improper use of a Context
	'''

	pass

class Context:
	'''
	Python context class

	This class implements a Stencila context on the Python side.
	It is called from within the extension module by the C++ PythonContext class.
	'''

	def __init__(self,namespace=None):
		if namespace is None: namespace = Namespace()
		self.namespaces = [namespace]

	# Shortcut methods for accessing the namespace stack
	
	def push(self,namespace=None):
		'''
		Push a new namespace on to the stack
		'''
		if namespace is None: namespace = Namespace(self.top())
		self.namespaces.append(namespace)
		return self

	def pop(self):
		'''
		Pop the current namespace off the top of the stack
		'''
		self.namespaces.pop(len(self.namespaces)-1)
		return self

	def top(self):
		'''
		Get the top of the namepace stack (i.e. the current namespace)
		'''
		return self.namespaces[len(self.namespaces)-1]

	def get(self,name):
		'''
		Get a variable from the top of the stack
		'''
		return self.top()[name]

	def set(self,name,value):
		'''
		Set a variable in the top of the stack
		'''
		self.top()[name] = value
		return self

	def evaluate(self,expression):
		'''
		Evaluate an expression in the top of the stack
		'''
		return eval(expression,{},self.top())

	# Context methods that provide the interface defined in cpp/stencila/contexts/context.hpp

	def execute(self,code):
		'''
		Execute some code
		'''
		exec code in self.top()
		return self

	def assign(self,name,expression):
		'''
		Assign an expression to a name
		'''
		self.top()[name] = self.evaluate(expression)
		return self

	def write(self,expression):
		'''
		Get a text representation of an expression
		'''
		return str(self.evaluate(expression))

	def paint(self,format,code):
		'''
		Get an image representation of some code
		'''
		raise NotImplementedError
		return self

	def test(self,expression):
		'''
		Test whether an expression is true or false
		'''
		return bool(self.evaluate(expression))

	def mark(self,expression):
		'''
		Mark an expression to be the subject of subsequent `match` queries
		'''
		self.enter()
		self.set('__subject__',self.evaluate(expression))
		return self

	def match(self,expression):
		'''
		Test whether an expression matches the current subject
		'''
		try:
			subject = self.get('__subject__')
		except KeyError:
			raise ContextError('No subject defined for switch directive')
		return subject==self.evaluate(expression)

	def unmark(self):
		'''
		Unmark the current subject
		'''
		self.exit()
		return self

	def begin(self,item,expression):
		'''
		Begin a for loop
		'''
		# Enter a new namespace
		self.enter()
		# Set variable __item__ to the name of each item
		# in the loop
		self.set('__item__',item)
		# Evaluate expression
		items = self.evaluate(expression)
		# Get iterator for this expression
		iterator = items.__iter__()
		# Set variable __items__ to iterator
		self.set('__items__',iterator)
		# Return next now (may be false if no item in expression)
		return self.next()

	def next(self):
		'''
		Step the current loop to the next item
		'''
		try:
			item = self.get('__items__').next()
		except StopIteration:
			return False
		else:
			self.set(self.get('__item__'),item)
			return True

	def leave(self):
		'''
		Leave a for loop
		'''
		self.exit()
		return self

	def enter(self,expression=None):
		'''
		Enter a new namespace
		'''
		if expression:
			self.push(self.evaluate(expression))
		else:
			self.push()
		return self

	def exit(self):
		'''
		Exit the current namespace
		'''
		self.pop()
		return self
