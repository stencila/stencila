# Stencila Web

**Components and clients for using Stencila from a web browser**

## 🛠️ Develop

### Testing

There are some HTML files for "manual" testing of components and clients. During development, you can launch these on localhost using Parcel e.g.

```console
npx parcel src/clients/test.html
```

> [!NOTE]
> The Parcel config currently uses `parcel/transformer-typescript-tsc` because of this [issue](https://github.com/parcel-bundler/parcel/issues/7425) related to decorators.

