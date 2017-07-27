# Data model for a Datatable

- a requirements/specifications document so that we can decide on the best approach to developing a XML schema for Datatables

- this document will either need to be updated to be consistent with the developed schema, or deleted

- initially, for determining approach, we should focus on the MUSTs and worry less about the OPTIONALs

- a Datatable is similar to database table, it has one or more **columns** (a.k.a _fields_) and for each column there is a value in each **row**

- although there is an emphasis on a column wise schema for Datatables this does not necessarily mean that the in-memory representation needs to be column-wise 

- a Datatable can have OPTIONAL meta data such as **name**, **title**, **description** and **authors** which the user SHOULD be able to add and modify

- Each **column**,

    - MUST have a **name**, this name SHOULD be able to be set and changed by the user, and/or be defined by the external data source (e.g. a CSV file with a header)

    - MUST have a **type** which specifies the _type_ of data that can be in that column:

        - the user SHOULD be able to change the type of a column

        - types might include string, number, integer, boolean, date

        - the default type is "string"

        - having types allows for editing restrictions on the type of data that a user can enter into a cell

        - having types allows for optimisations in storage and analysis across the Datatable

    - can have one or more OPTIONAL **constraints** which specifies the acceptable _values_ in each column

    - can have an OPTIONAL **title** string which the user SHOULD be able to modify

    - can have an OPTIONAL **description** string which the user SHOULD be able to modify

- Similarity and difference to `Sheets`:

  - Superficially the UI for `Datatables` is similar to that for `Sheets` in that they are both grid based

  - However, the _data model_ is quite different. Sheets can be though of as a graph of cells, each specifying their dependency on other cells via cell formulae. In a spreadsheet those cells are displayed in a grid. But that is really a UI convention, not a reflection of the underlying data model. The position of a Sheet cell in that sheet does not determine the data content of that cell. A cell in column A1 is not restricted to having the same type of value as a cell in A2 even though they are in the same column. In contrast, in a Datatable, cells in a column are restricted to having the same type.

  - A Sheet tends to be more sparse having more empty cells. It may be best to model sparse Sheets as a graph of cells having a particular position e.g. `A1` rather than them being either column or row oriented data

  - Where sheets are dense, and thus more suitable to row or column based operations, that is usually because they contain a lot of data - in those cases we would recommend that the user use a Datatable for that data
