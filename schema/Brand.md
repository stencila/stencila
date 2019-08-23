---
title: Brand
authors: []
---

include: ../public/Brand.schema.md
:::
A brand is a name used by an organization or business person for labeling a product, product group, or similar. https&#x3A;//schema.org/Brand.

| Entity | type           | The name of the type and all descendant types.                               | string |
| ------ | -------------- | ---------------------------------------------------------------------------- | ------ |
| Entity | id             | The identifier for this item.                                                | string |
| Thing  | alternateNames | Alternate names (aliases) for the item.                                      | array  |
| Thing  | description    | A description of the item.                                                   | string |
| Thing  | meta           | Metadata associated with this item.                                          | object |
| Thing  | name           | The name of the item.                                                        | string |
| Thing  | url            | The URL of the item.                                                         | string |
| Brand  | logo           | A logo of of the brand. It can be either a URL of the image or image itself. |        |
|        |                |                                                                              |        |
| Brand  | reviews        | Short reviews of the brand and/or the products it represents.                |        |
| array  |                |                                                                              |        |

:::

The `Brand` type allows you to provide details about a brand such as its logo and reviews. This type of often used to describe the `brand` of a [`Product`](/Product).

# Examples

This is a `Brand` with a logo and one review.

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
