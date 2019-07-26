---
title: Media Object
authors: []
---

include: ../built/MediaObject.schema.md
:::
A media object, such as an image, video, or audio object embedded in a web page or a downloadable dataset. https&#x3A;//schema.org/MediaObject

| Entity       | type           | The name of the type and all descendant types.                                | string |
| ------------ | -------------- | ----------------------------------------------------------------------------- | ------ |
| Entity       | id             | The identifier for this item.                                                 | string |
| Thing        | alternateNames | Alternate names (aliases) for the item.                                       | array  |
| Thing        | description    | A description of the item.                                                    | string |
| Thing        | meta           | Metadata associated with this item.                                           | object |
| Thing        | name           | The name of the item.                                                         | string |
| Thing        | url            | The URL of the item.                                                          | string |
| CreativeWork | authors        | The authors of this this creative work.                                       | array  |
| CreativeWork | citations      | Citations or references to other creative works, such as another publication, |        |

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | MediaObject | bitrate | Bitrate in megabits per second (Mbit/s, Mb/s, Mbps). | number | | MediaObject | contentSize | File size in megabits (Mbit, Mb). | number | | MediaObject | contentUrl | URL for the actual bytes of the media object, for example the image file or video file. | string | | MediaObject | embedUrl | URL that can be used to embed the media on a web page via a specific media player. | string | | MediaObject | format | Media type (MIME type) as per http&#x3A;//www.iana.org/assignments/media-types/media-types.xhtml. | string |
:::

The `MediaObject` type allows you to provide details about the kinds of `CreativeWork` items that are in a video, image or audio format. These details can be: bitrate, size, url, encoding formate, embedded url.

## Examples

This is a simple `MediaObject` representing an MP3 file that sources content from a URL.

```json
{
  "type": "MediaObject",
  "bitrate": "44",
  "contentSize": "2",
  "contentUrl": "http://www.example.com/file.mp3",
  "encodingFormat": "audio/mpeg3",
  "embedUrl": "http://www.example.com/full_size/file.mp3"
}
```
