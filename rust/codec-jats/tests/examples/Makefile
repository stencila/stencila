all: biorxiv-573522.jats.xml \
     biorxiv-630838.jats.xml \
	 elife-94334-v1.jats.xml \
	 pmc-6849754.jats.xml \
	 pmc-8438044.jats.xml \
	 pmc-9786276.jats.xml \
	 format

biorxiv-573522.jats.xml:
	curl -sSL https://www.biorxiv.org/content/early/2024/01/01/2023.12.31.573522.source.xml -o $@

biorxiv-630838.jats.xml:
	curl -sSL https://www.biorxiv.org/content/early/2025/01/01/2024.12.31.630838.source.xml -o $@

elife-%.jats.xml:
	curl -sSL https://raw.githubusercontent.com/elifesciences/elife-article-xml/a68557ed7a66b02c454f264e1dc0a817368aeef4/articles/elife-$*.xml -o $@

pmc-%.jats.xml:
	curl -sSL "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pmc&rettype=xml&id=PMC$*" -o $@

format:
	npx prettier --plugin=@prettier/plugin-xml --xml-whitespace-sensitivity preserve --write '**/*.jats.xml'
