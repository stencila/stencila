import esbuild from 'esbuild'

const ctx = await esbuild.context({
  entryPoints: ['src/extension.ts'],
  bundle: true,
  format: 'cjs',
  minify: true,
  sourcemap: false,
  sourcesContent: false,
  platform: 'node',
  outfile: 'out/extension.js',
  external: ['vscode'],
  logLevel: 'warning',
});

await ctx.rebuild();
await ctx.dispose();
