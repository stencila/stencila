# Stencila Sheet schema

Documentation for the spreadsheet XML schema defined in `SpreadsheetSchema.rng`. Put here, instead of as `<a:documentation>` elements in the RNG, because it makes both the schema and the documentation, more readable.

Where appropriate we have re-used existing schema definitions for spreadsheets and tabular data including:

- [metadata of the Frictionless Data Data Resource](https://specs.frictionlessdata.io/data-resource/#metadata-properties).
- [field descriptors the Frictionless Data Table Schema](https://specs.frictionlessdata.io/table-schema/#field-descriptors)

- `sheet` element

  Contains one each of `meta` and `data` elements.

  - `meta` element

    Contains one each of `name`, `title`, `description` and `columns` elements (in any order).

      - `name` element

        Text name for the spreadsheet.

      - `title` element

        Text title for the spreadsheet.

      - `description` element

        Text description of the spreadsheet.

      - `columns` element

        Contains zero or more `col` elements.

        - `col` element

          Empty element with optional `type` and `format` attribute.

            - `type` attribute

              Specifies the expected type of the cell values in the column (or individual cell).

              Choice of: `string`, `number`, `integer`, `boolean`, `object`, `array`, `date`, `time`, `datetime`, `year`, `yearmonth`, `duration`, `geopoint`, `geojson`, `any`

            - `format` attribute

              Specifies the format to be applied to the cell values in the column (or individual cell).

              A `sprinf` string (?).

  - `data` element

    Contains zero or more `row` elements

    - `row` element

      Has a `height` attribute and zero or more `cell` elements.

      - `cell`

      Text value and optional `type` and `format` attributes (see `col` element above).
