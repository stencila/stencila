# ImageObject

The `ImageObject` type allows you to provide details about an image file (in any format) such as its caption, thumbnail and further information inherited from [`MediaObject`](/MediaObject).

## Examples

```json
{
  "type": "ImageObject",
  "caption": "Kiwi bird",
  "contentSize": "1.3 MB",
  "contentUrl": "http://www.example.com/kiwiBird.png",
  "encodingFormat": "image/png",
  "thumbnail": {
    "type": "ImageObject",
    "contentUrl": "http://www.example.com/kiwiBird_mini.png"
  }
}
```
