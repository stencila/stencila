# Creative Work

The most generic kind of creative work, including books, movies, photographs, software programs, etc.

## Examples

For example usage see:

- [`Article`](/schema/Article)
- [`Collection`](/schema/Collection)
- [`Datatable`](/schema/Datatable)
- [`Media Object`](/schema/MediaObject)
- [`Software Application`](/schema/SoftwareApplication)
- [`Table`](/schema/Table)

```json
{
"type": "CreativeWork",
"authors":[
    {"type": "Person",   "givenNames": ["Marie"], "familyNames": ["Skłodowska", "Curie"]}
    ],
"title": "Radioactivity",
"citations": ["Marie Curie's century-old radioactive notebook still requires lead box"],
"content":[
     {"type": "Node"} ],
"datePublished": "	2019-05-20",
"editors":[
    {"type": "Person",   "givenNames": ["John"], "familyNames": ["Smith"]}
    ],
"funders": [
    {"type": "Person",   "givenNames": ["Joanna"], "familyNames": ["Smith"]}
    ],
"isPartOf": {
    "type": "CreativeWork",
    "title": "The Great Works of Marie Skłodowska-Curie"
},
"licenses":[
    {"https://opensource.org/licenses/MIT"}
],
"publisher": "Random House",
"text": "Sample text",
"version": "4"
}
```
