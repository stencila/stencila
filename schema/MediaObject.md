# MediaObject

The `MediaObject` type allows you to provide details about the kinds of `CreativeWork` items that are in a video, image or audio format. These details can be: bitrate, size, url, encoding formate, embedded url.

## Examples

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
