all: biorxiv-573522.xml elife-94334-v1.xml format

# bioXiv articles

# First article listed at https://api.biorxiv.org/details/biorxiv/2024-01-01/2024-01-01/1
biorxiv-573522.xml:
	curl -sSL https://www.biorxiv.org/content/early/2024/01/01/2023.12.31.573522.source.xml -o $@

# eLife articles

# Article last committed to elifesciences/elife-article-xml repo as of 2024-12-11T09:56:56Z
elife-94334-v1.xml:
	curl -sSLO https://raw.githubusercontent.com/elifesciences/elife-article-xml/a68557ed7a66b02c454f264e1dc0a817368aeef4/articles/elife-94334-v1.xml

format:
	npx prettier --plugin=@prettier/plugin-xml --xml-whitespace-sensitivity preserve --write '**/*.xml'
