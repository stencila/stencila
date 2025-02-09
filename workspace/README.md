# Stencila Workspace

**A Docker image for using Stencila with OpenVSCode Server**

This image is based on [`gitpod-io/openvscode-server`](https://github.com/gitpod-io/openvscode-server) and pre-installs:

- [Stencila VSCode extension](https://marketplace.visualstudio.com/items?itemName=stencila.stencila)

- Other useful VSCode extensions such as [Code Spell Checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker) and [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens)

- [GitHub CLI](https://cli.github.com/)

- Python and tooling e.g. [`uv`](https://docs.astral.sh/uv/) and [`ruff`](https://docs.astral.sh/ruff/)

- Commonly used Python packages such as `pandas`, `polars`, `matplotlib`, and `plotly`.

- R and tooling e.g. [`lintr`](https://github.com/r-lib/lintr/) and [`styler`](https://github.com/r-lib/styler)

- Commonly used R packages such as `tidyverse` and `data.table`.