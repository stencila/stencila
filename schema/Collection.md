# Collection

The `Collection` type allows you to represent a collection of various `CreativeWork` derrived types, such as [`Article`](/Article), [`Datatable`](/Datatable) or [`Table`](/Table). These parts of the `Collection` are a required property.


## Example

```json
{
    "type": "Collection",
    "editors": [
         {"type": "Person",   "givenNames": ["John"], "familyNames": ["Smith"]}
    ],
    "parts": [
        {"type": "Article",  "title": "Recherches sur les substances radioactives"},
        {"type": "Table", "rows": [
            {"type":"TableRow", "cells": [{"type": "TableCell","content": [1]}] } 
            ]
        }
    ]    
}
```
