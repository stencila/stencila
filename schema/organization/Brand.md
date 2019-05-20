# Brand

The `Brand` type allows you to provide details about a brand such as its logo and reviews. This type of often used to describe the `brand` of a [`Product`](/Product).

## Example

```json
{
  "type": "Brand",
  "name": "XYZ",
  "logo": {
    "type": "ImageObject",
    "url": "https://example.com/xyz.png",
    "caption": "Logo of Brand YXZ"
  },
  "reviews": ["Rather average product. Not sure if would use again"]
}
```
