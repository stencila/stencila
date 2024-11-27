# Developing and testing

To test the extension in VSCode, you will need `cargo` and `node` installed. Ensure JavaScript dependencies are installed using,

```sh
npm install
```

Then, build the extension's JavaScript and `stencila` CLI binary (which contains the Stencila Language Server used by the extension using):

```sh
npm run compile
```

Finally, within VSCode, press `F5` to run the "VSCode Extension" debug task which will bring up an "Extension Development Host" with the Stencila extension loaded.

There is also a test suite which can be run using

```sh
npm test
```

If you prefer, there is also a `Makefile` with recipes for these and other tasks e.g.

```sh
make fix test
```

> [!NOTE] Not in root NPM workspace
>
> This package can not be part of the root NPM workspace in this
> repository as that causes issues with packaging (trust me, I tried :/)


# Contributing to `README.md`

The `README.md` in this directory is the "font page" for the extension visible in the VSCode Marketplace and Open VSX Registry. 

Links to images, can not be relative, they must use `https://` URLs. Ordinarily, relative image links would work but does not for this extension because it is within a subdirectory of the repository. We have tried numerous things to deal with this including adding the `repository.directory` property in `package.json` and using the `--baseContentUrl` and `--baseImageUrl` options to `vsce publish`. Nothing worked other than just using absolute URLs.
