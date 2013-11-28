from bs4 import BeautifulSoup

# A list of elements that will usually be excluded from output
# You don't have to use this list (see render() below)
blacklist = [
	# Root element - exclude all
	'html',
	# Document metadata - exclude all
	'head','title','base','link','meta','style',
	# Sections - exclude body
	'body',
	# Scripting
	'script','noscript',
	# Embedded content - exclude all except <math>
	'img','iframe','embed','object','param','video','audio','source','track','canvas','map','area','svg',
	# Forms - exclude all
	'form','fieldset','legend','label','input','button','select','datalist','optgroup','option','textarea','keygen','output','progress','meter',
	# Interactive elements - exclude all
	'details','summary','menuitem','menu',
]

def element(name,desc):
	'''
	A shortcut function to return an element dictionary
	'''
	return dict(
		name = name,
		desc = desc,
	)

def elements_mdn():
	'''
	Parses and exracts a list of elements from a file downloaded from
		https://developer.mozilla.org/en-US/docs/Web/Guide/HTML/HTML5/HTML5_element_list
	'''
	soup = BeautifulSoup(file('mozilla-mdn-html5-element-list.html').read())
	elements = []
	for a in soup.find_all('a'):
		if '/en-US/docs/Web/HTML/Element/' in a.get('href'):
			href = a.get('href')
			name = href.replace('/en-US/docs/Web/HTML/Element/','')
			desc = a.get('title').encode("ascii","ignore")
			elements.append(element(name,desc))
	# The h1,h2...etc tags are not parse by the above so add them manually
	for name in 'h1','h2','h3','h4','h5','h6':
		elements.append(element(
			name,
			'Heading element <%s>. A heading element briefly describes the topic of the section it introduces.'%name
		))
	return elements

def render(template_filename,elements,excludes):
	'''
	Renders a template for each element not in excludes
	'''
	template = file(template_filename).read()
	output = file(template_filename+'.out','w')

	for element in elements:
		if not (element['name'] in excludes):
			print>>output, template%element
			output.flush()

if __name__=='__main__':
	elements = elements_mdn()
	render('html5-list.txt',elements,blacklist)
	render('r-stencil-inline.txt',elements,blacklist)
