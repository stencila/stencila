import { css } from 'lit'

/**
 * Global style rules for tables and images within the outputs,
 * Override, or append to the `LitElement.styles` property in any view.
 */
export const outputCSS = css`
  /* Keep all img tags within their parent's bounds */

  img {
    object-fit: fill;
    width: 100%;
    display: block;
  }

  table {
    border-spacing: 0px;
  }

  th {
    padding: 1rem;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  td {
    padding: 0.5rem 1rem;
  }

  /* Align numeric cells to the right */
  td[data-type='number'],
  td[data-type='integer'] {
    text-align: right;
  }

  figure {
    margin: 0;
  }
`
