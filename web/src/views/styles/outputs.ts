import { css } from 'lit'

/**
 * Global style rules for tables and images within the outputs,
 * Override, or append to the `LitELement.styles` property in any view.
 */
const outputCSS = css`
  /* keep all img tags within there parent's bounds */
  img {
    height: auto;
    width: 100%;
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

  /* align numerical values to the right */
  td[data-type='number'],
  td[data-type='integer'] {
    text-align: right;
  }
`

export { outputCSS }
