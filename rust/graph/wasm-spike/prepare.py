"""Copy the unchanged analysis passes into a disposable standalone spike."""
from pathlib import Path
import re
import argparse
import shutil

parser = argparse.ArgumentParser()
parser.add_argument('--omit', choices=['python','r','javascript','typescript','julia','matlab','rust','snakemake'])
args = parser.parse_args()
spike = Path(__file__).resolve().parent
graph = spike.parent
src = spike / 'src'
(src / 'code').mkdir(parents=True, exist_ok=True)
for name in ['analyze', 'ast', 'facts', 'io_table', 'language', 'normalize', 'resolve', 'util']:
    text = (graph / 'src/code' / f'{name}.rs').read_text()
    if name == 'language':
        text = text.replace('stencila_codecs::Format', 'stencila_format::Format')
        if args.omit:
            grammar = args.omit
            constant = 'LANGUAGE_TYPESCRIPT' if grammar == 'typescript' else 'LANGUAGE'
            text = text.replace(f'tree_sitter_{grammar}::{constant}.into()', 'unreachable!("grammar omitted for size measurement")')
    if name == 'facts':
        # Serialize ALL fields, including private bookkeeping, for exhaustive parity.
        text = re.sub(r'#\[derive\(([^\n]*)\)\]', r'#[derive(serde::Serialize, \1)]', text)
    (src / 'code' / f'{name}.rs').write_text(text)
shutil.copytree(graph / 'src/code/scan', src / 'code/scan', dirs_exist_ok=True)
shutil.copytree(graph / 'rules', spike / 'rules', dirs_exist_ok=True)
(src / 'code.rs').write_text(''.join(f'mod {name};\n' for name in ['analyze','ast','diagnostics','facts','io_table','language','normalize','resolve','scan','util']) + '\npub use analyze::analyze_source;\npub use language::CodeLanguage;\n')
text = (graph / 'src/code/diagnostics.rs').read_text()
text = text[text.index('/// The generic remedy'):text.index('/// One unresolved I/O operation')]
text = text.replace('#[derive(Debug,', '#[derive(serde::Serialize, Debug,')
(src / 'code/diagnostics.rs').write_text(text)
text = (graph / 'src/package.rs').read_text()
text = text[text.index('/// A package imported'):text.index('/// Create the graph-local id')]
text = text.replace('#[derive(Debug,', '#[derive(serde::Serialize, Debug,')
(src / 'package.rs').write_text(text)
text = (graph / 'src/reference.rs').read_text()
text = text[text.index('/// Check whether a reference has a URI'):text.index('/// Check whether a reference uses a scheme')]
(src / 'reference.rs').write_text(text)
# Keep the repository's resolved versions rather than updating dependencies.
if not (spike / 'Cargo.lock').exists():
    shutil.copyfile(graph.parents[1] / 'Cargo.lock', spike / 'Cargo.lock')
