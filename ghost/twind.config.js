import { setup } from 'twind';

setup({
  theme: {
    extend: {
      colors: {
        customBlue: '#3498db',
      }
    }
  },
  preflight: true, // Enables Tailwind's base styles
  mode: 'silent',  // Prevents errors for unknown classes
  rules: [
    ['some-random-class', { color: 'purple' }], // Custom class
    ['my-custom-style', { background: 'yellow' }] // Custom class
  ]
});
