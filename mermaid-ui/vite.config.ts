import { defineConfig } from 'vite';

import dts from 'vite-plugin-dts';

export default defineConfig({
  build: {
    lib: {
      entry: 'src/index.ts', // Path to your entry file
      name: 'mermaid', // Name of the exported global variable
      fileName: (format) => `index.${format}.js`, // Output filename
      formats: ['es'] // Generate ES module and UMD formats
    },
    rollupOptions: {
      // external: ['mermaid'], // Treat mermaid as an external dependency
      output: {
        globals: {
          mermaid: 'mermaid' // Map the global variable for UMD build
        },
          // Optional: specific configuration for .mjs output
        entryFileNames: '[name].mjs',
        chunkFileNames: '[name]-[hash].mjs'
      }
    }
  },
  plugins: [
    dts() // Optional: generates .d.ts files
  ]
});

// export default defineConfig({
//   build: {

//     rollupOptions: {
//       external: ['mermaid'], // Treat mermaid as an external dependency
//       output: {
//         globals: {
//           mermaid: 'mermaid' // Map the global variable for UMD build
//         }
//       }
//     }
//   },
//   plugins: [
//     dts() // Optional: generates .d.ts files
//   ]
// });
