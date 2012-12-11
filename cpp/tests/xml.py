from lxml.cssselect import CSSSelector

sel = CSSSelector('div[a^=r]')
print sel.path
