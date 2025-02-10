import { tw } from 'twind';

/**
 * Process the tw css utility classes at runtime for the raw blocks and styled blocks.
 * 
 * This is to be used in servless mode only, for static web publications such as ghost
 */
export default function processTailwindAtRuntime() {
    // Define custom elements to target
    const rawBlocks = document.querySelectorAll('stencila-raw-block');

    if (rawBlocks.length) {

      rawBlocks.forEach(element => {
        const slot = element.shadowRoot?.querySelector('slot[name="content"]');
        if (!slot) {
          return
        };
        requestAnimationFrame(() => {
          const assignedNode = slot.assignedNodes()[0]

          assignedNode.querySelectorAll('[class]').forEach(el => {
              el.className = tw(el.className); // Process Tailwind classes correctly
          });
        });
      });
    };

    const styleBlocks = document.querySelectorAll('stencila-styled-block');
    if (styleBlocks.length) {
      styleBlocks.forEach((block) => {
        if (block.hasAttribute('code')) {
          const content = block.querySelector('[slot="content"]');
          if (content) {
            content.className = tw(block.getAttribute('code'));
          };
        };
      });
    };
};