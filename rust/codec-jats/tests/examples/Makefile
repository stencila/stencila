all: biorxiv-573522.jats.xml biorxiv-630838.jats.xml elife-94334-v1.jats.xml pmc-6849754.jats.xml format

# bioXiv articles

# First article listed at https://api.biorxiv.org/details/biorxiv/2024-01-01/2024-01-01/1
biorxiv-573522.jats.xml:
	curl -sSL https://www.biorxiv.org/content/early/2024/01/01/2023.12.31.573522.source.xml -o $@

# First article listed at https://api.biorxiv.org/details/biorxiv/2025-01-01/2025-01-01/1
biorxiv-630838.jats.xml:
	curl -sSL https://www.biorxiv.org/content/early/2025/01/01/2024.12.31.630838.source.xml -o $@

# eLife articles

# Article last committed to elifesciences/elife-article-xml repo as of 2024-12-11T09:56:56Z
elife-94334-v1.jats.xml:
	curl -sSLO https://raw.githubusercontent.com/elifesciences/elife-article-xml/a68557ed7a66b02c454f264e1dc0a817368aeef4/articles/elife-94334-v1.xml

# PMC articles

# Article with inline and block MathML
pmc-6849754.jats.xml:
	curl -sSL "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&id=PMC6849754&rettype=xml" -o $@

format:
	npx prettier --plugin=@prettier/plugin-xml --xml-whitespace-sensitivity preserve --write '**/*.jats.xml'
