import { tw, setup } from 'twind';

import { getStyleTag, virtualSheet } from 'twind/sheets'

// Create a virtual sheet that captures styles without injecting them globally
const sheet = virtualSheet()

setup({
  preflight: false,
  mode: 'silent',
  sheet: sheet
});

/**
 * Process the tw css utility classes at runtime for the raw blocks and styled blocks.
 * 
 * This is to be used in servless mode only, for static web publications such as ghost
 */
export default function processTailwindAtRuntime() {
  sheet.reset()

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
            el.className = tw(el.className);
        });

        const tag = document.createElement('style')

        tag.textContent = sheet.target.map((rule) => {
          return `stencila-raw-block [slot="content"] ${rule}`
        }).join('\n')

        document.head.appendChild(tag)
      });
    });
  };

  const styleBlocks = document.querySelectorAll('stencila-styled-block');
  if (styleBlocks.length) {
    styleBlocks.forEach((block) => {
      if (block.hasAttribute('code')) {
        const content = block.querySelector('[slot="content"]');
        if (content) {
          requestAnimationFrame(() => {
            console.log('hi')
            content.className = tw(block.getAttribute('code'));
          
            const tag = document.createElement('style')

            tag.textContent = sheet.target.map((rule) => {
              return `stencila-styled-block [slot="content"]${rule}`
            }).join('\n')

            document.head.appendChild(tag)
          });
        }
      }
    });
  };


};