# AudioObject

The `AudioObject` type allows you to provide details such as caption and transcript, and the details which are included in [`MediaObject`](/MediaObject) which `AudioObject` extends.

## Examples

```json
{
  "type": "AudioObject",
  "caption": "Example Audio File",
  "contentSize": "5 Mb",
  "contentUrl": "http://www.example.com/file.mp3",
  "encodingFormat": "audio/mpeg3",
  "transcript": "This is the transcript for the audio file..."
}
```
