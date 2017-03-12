## Stencila

Stencila is a platform for creating, collaborating on, and sharing data driven content. Content that is transparent and reproducible, like RMarkdown and Jupyter. Content that can be versioned and composed just like we do with open source software using tools like CRAN and NPM. And above all, content that is accessible to non-coders, like Google Docs and Microsoft Office.

### Development

```
git clone https://github.com/stencila/ui.git
cd ui
npm install
npm run start
```

Now you can access the different interfaces in the browser:

- http://localhost:4000/examples/document
- http://localhost:4000/examples/sheet


Most development tasks can be run directly using Javascript tooling (`npm`)

Task                                                    | Command               |
------------------------------------------------------- |-----------------------|
Install and setup dependencies                          | `npm install`         |
Run the development server                              | `npm start`           |
Clean and rebuild                                       | `npm run build`       |


### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/ui/issues/new), add to [existing issues](https://github.com/stencila/ui/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.
