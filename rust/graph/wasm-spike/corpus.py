"""Collect all supported source fixtures and embedded document code, plus benchmarks."""
from pathlib import Path
import json
import re

root = Path(__file__).resolve().parent
fixtures = root.parent / 'tests/fixtures'
aliases = {'py':'python','r':'r','jl':'julia','m':'matlab','js':'javascript','ts':'typescript','rs':'rust','smk':'snakemake','nf':'nextflow'}
cases = []

def add(path, language, source):
    cases.append({'path':path, 'language':language, 'source':source})

def document_code(node, path):
    if isinstance(node, dict):
        if node.get('type') in ['CodeChunk', 'CodeExpression']:
            language = node.get('programmingLanguage', '').lower()
            language = aliases.get(language, language)
            add(path + '#' + node.get('id', str(len(cases))), language, node['code'])
        for value in node.values():
            document_code(value, path)
    elif isinstance(node, list):
        for value in node:
            document_code(value, path)

for path in sorted(fixtures.rglob('*')):
    if not path.is_file():
        continue
    logical = str(path.relative_to(fixtures))
    language = 'snakemake' if path.name == 'Snakefile' else ('nextflow' if path.name == 'nextflow.config' else aliases.get(path.suffix[1:].lower()))
    if language:
        add(logical, language, path.read_text())
    elif path.suffix == '.json':
        try:
            document_code(json.loads(path.read_text()), logical)
        except json.JSONDecodeError:
            # The negative fixture intentionally includes an invalid package.json.
            print(f'Skipping malformed non-code JSON fixture: {logical}')
    elif path.suffix == '.smd':
        for index, match in enumerate(re.finditer(r'(?m)^([ \t]*)```(\w+)[^\n]*\n(.*?)^\1```', path.read_text(), re.S)):
            language = aliases.get(match[2].lower(), match[2].lower())
            if language in aliases.values():
                lines = match[3].splitlines(keepends=True)
                source = ''.join(line[len(match[1]):] if line.startswith(match[1]) else line for line in lines)
                add(f'{logical}#fence-{index}', language, source)

representatives = {
    'python': 'code-polyglot-tools-io/analysis.py',
    'r': 'code-polyglot-tools-io/analysis.R',
    'javascript': 'code-polyglot-tools-io/analysis.js',
    'typescript': 'code-polyglot-tools-io/analysis.ts',
    'julia': 'code-polyglot-tools-io/analysis.jl',
    'matlab': 'code-polyglot-tools-io/analysis.m',
    'rust': 'code-polyglot-tools-io/src/bin/analysis.rs',
    'snakemake': 'code-workflow-snakemake-rnaseq/Snakefile',
    'nextflow': 'code-workflow-nextflow-metagenomics/main.nf',
}
benchmarks = []
for language, path in representatives.items():
    case = next(case for case in cases if case['path'] == path)
    benchmarks.append(dict(case, kind='representative'))
    if language == 'rust':
        lines = ['fn main() {'] + [f'let value_{i} = {i};' if i % 25 else f'let value_{i} = std::fs::read_to_string("data/input.csv");' for i in range(4998)] + ['}']
    elif language == 'snakemake':
        lines = []
        for i in range(1000):
            lines.extend([f'rule task_{i}:', f'    input: "data/input-{i}.csv"', f'    output: "results/output-{i}.csv"', '    shell: "python transform.py {input} {output}"', ''])
    elif language == 'nextflow':
        lines = []
        for i in range(1000):
            lines.extend([f'process TASK_{i} {{', 'input:', 'path reads', 'output: path "result.csv"', '}'])
    else:
        assignment = {
            'python': lambda i: f'value_{i} = {i}' if i % 25 else f'value_{i} = open("data/input.csv")',
            'r': lambda i: f'value_{i} <- {i}' if i % 25 else f'value_{i} <- read.csv("data/input.csv")',
            'javascript': lambda i: f'const value_{i} = {i};' if i % 25 else f'const value_{i} = readFileSync("data/input.csv");',
            'typescript': lambda i: f'const value_{i}: number = {i};' if i % 25 else f'const value_{i} = readFileSync("data/input.csv");',
            'julia': lambda i: f'value_{i} = {i}' if i % 25 else f'value_{i} = CSV.read("data/input.csv", DataFrame)',
            'matlab': lambda i: f'value_{i} = {i};' if i % 25 else f'value_{i} = readtable("data/input.csv");',
        }[language]
        lines = [assignment(i) for i in range(5000)]
    benchmarks.append({'path':f'generated/{language}-5000', 'language':language, 'source':'\n'.join(lines) + '\n', 'kind':'large'})
(root / 'corpus.json').write_text(json.dumps({'fixtures':cases, 'benchmarks':benchmarks}, indent=2) + '\n')
print(f'Collected {len(cases)} fixture sources and {len(benchmarks)} benchmark cases')
