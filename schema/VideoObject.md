---
title: VideoObject
authors: []
---

include: ../public/VideoObject.schema.md
:::
A video file. https&#x3A;//schema.org/VideoObject

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

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | MediaObject | bitrate | Bitrate in megabits per second (Mbit/s, Mb/s, Mbps). | number | | MediaObject | contentSize | File size in megabits (Mbit, Mb). | number | | MediaObject | contentUrl | URL for the actual bytes of the media object, for example the image file or video file. | string | | MediaObject | embedUrl | URL that can be used to embed the media on a web page via a specific media player. | string | | MediaObject | format | Media type (MIME type) as per http&#x3A;//www.iana.org/assignments/media-types/media-types.xhtml. | string | | VideoObject | caption | The caption for this video recording. | string | | VideoObject | thumbnail | Thumbnail image of this video recording. | | | VideoObject | transcript | The transcript of this video recording. | string |
:::

The `VideoObject` type allows you to provide details such as caption and transcript, and the details which are included in [`MediaObject`](/MediaObject) which `VideoObject` extends.

# Examples

This is a simple `VideoObject` representing an MP4 file that sources content from a URL.

```json
{
  "type": "VideoObject",
  "caption": "Example Video File",
  "contentSize": "45",
  "contentUrl": "http://www.example.com/file.mp4",
  "encodingFormat": "video/mpeg",
  "transcript": "This is the transcript for the video file..."
}
```
