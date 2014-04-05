import inspect

class Namespace(dict):
	'''
	A simple namespace class used by the Context class
	'''

	def __init__(self,parent=None):
		if parent==None: parent = inspect.currentframe(1).f_locals
		self.parent = parent

	def __getitem__(self,name):
		try:
			return dict.__getitem__(self, name)
		except KeyError:
			if self.parent: return self.parent[name]
			else: raise KeyError(name)
