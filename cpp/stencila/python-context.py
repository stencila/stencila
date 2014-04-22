R"(

import inspect

class __stencila__namespace__(dict):

	def __init__(self,parent=None):
		if parent==None: parent = inspect.currentframe(1).f_locals
		self.parent = parent

	def __getitem__(self,name):
		try:
			return dict.__getitem__(self, name)
		except KeyError:
			if self.parent: return self.parent[name]
			else: raise KeyError(name)

class __stencila__context__:

	def __init__(self,namespace=None):
		if namespace is None: namespace = __stencila__namespace__()
		self.namespaces = [namespace]

	# Shortcut methods for accessing the namespace stack
	
	def push(self,namespace=None):
		# Push a new namespace on to the stack
		if namespace is None: namespace = __stencila__namespace__(self.top())
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

	def execute(self,code):
		exec code in self.top()

	def assign(self,name,expression):
		self.top()[name] = self.evaluate(expression)

	def write(self,expression):
		return str(self.evaluate(expression))

	def paint(self,format,code):
		raise NotImplementedError

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
			item = self.get('__items__').next()
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

)"
