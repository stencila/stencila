# Installing Stencila

You can run Stencila on Windows, Mac OS and Linux. There are two main ways to deploy Stencila on your machine: Stencila Desktop or Stencila Docker image.

If you decide to install Stencila Desktop, then you also need to enable the execution context, you want to run (R, Python, Javascript, SQL and so on). You can do that through installing relevant [Stencila packages](#execution-contexts). If you decide to use the Stencila Docker image, you do not have to install separately the execution contexts. However, you need to have [Docker](https://docs.docker.com/install/) installed on your machine.


## Stencila Desktop :sparkles:

<p class="tip">Stencila Desktop edition is still in under development. You can install it and run it using the built-in
[Mini language](languages/mini/README.md) which will allow you do some standard data manipulation. The current Stencila Desktop releases will not work with other
ececution contexts (R, Python, SQL, Javascript). If you want to try them out, you need to build Stencila Desktop on your
machine from the source.</p>

The Stencila Desktop is the most straightforward way to work with Stencila Documents. You can run it on Windows, Mac OS and Linux.

 * **Download for Windows** <br/>
 Download the `.exe` file for the latest [Stencila Desktop release]( https://github.com/stencila/desktop/releases). Then just double click downloaded file.

* **Download for Mac OS** <br />
Download the `.dmg` file for the latest [Stencila Desktop release]( https://github.com/stencila/desktop/releases). Then:
 - double click the `.dmg` to make its content available (the name will show up in the Finder sidebar, at the bottom), a pop-up window will open;
 - drag the application from the `.dmg` window into /Applications to launch install (you may need an administrator password);
 - wait for the copy process to finish;
 - eject the `.dmg` file by clicking the eject button in the Finder sidebar.

(Thanks to [patrix](https://apple.stackexchange.com/a/64848) for these instructions.)

* **Download for Linux** <br />
Download the AppImage\* file for the latest [Stencila Desktop release]( https://github.com/stencila/desktop/releases). Then double click on it and
click *yes* to "Make executable and run".

To install from command line, navigate to the folder where the AppImage file is located and then:

```bash
$ chmod a+x stencila-desktop-*.AppImage
$ ./stencila-desktop-*.AppImage
```
   \*[AppImage](http://appimage.org/) is a format for distributing applications for Linux and, on most distributions, will not require the installation of any dependencies.

### Execution contexts

**R context**

If you want to use R code within Stencila Documents working on Stencila Desktop, you need to enable the R Execution Context. It is done though installing the
[Stencila R package](https://github.com/stencila/r).
Because it is not yet available on CRAN, you need to install it using the [`devtools`](https://github.com/hadley/devtools) package from our Github repository.
Launch an R session on your machine and type:

```r
devtools::install_github("stencila/r")
```
Then install the package's manifest so that it can be found by the Stencila Desktop and Stencila packages for other languages,

```r
stencila:::install()
```

Installing Stencila R package will also enable the SQL execution context.

**Python context**

If you want to use Python code within Stencila Documents working on Stencila Desktop, you need to enable the Python Execution Context. It is done though installing the
[Stencila Python package](https://github.com/stencila/py). Because the Stencila Python package is not available via PyPI (Python Package Index) yet, you need to
install it using `pip` from our Github repository:

* on **Linux** and **Mac OS**, open your terminal and type:

```bash
pip install --user https://github.com/stencila/py/archive/master.zip
```

Then launch a Python session and register the package's manifest so that it can be found by the Stencila Desktop and Stencila packages for other languages,

```python
import stencila
stencila.install()
```

* on **Windows**
You need to enable `pip` on Windows. Please [follow these instructions](https://dev.to/el_joft/installing-pip-on-windows). Open the Windows terminal
(go to Start and then in "Search for programmes and files" type `cmd`). Then type in:

```bash
pip install --user https://github.com/stencila/py/archive/master.zip
```

Then launch a Python session and register the package's manifest so that it can be found by the Stencila Desktop and Stencila packages for other languages,

```python
import stencila
stencila.install()
```

Installing Stencila Python package will also enable the SQL execution context.

**SQL context**

If you want to use R code within Stencila Documents working on Stencila Desktop, you need to enable the R Execution Context. It is done though installing either
[Stencila R package](https://github.com/stencila/r) or [Stencila Python package](https://github.com/stencila/py).

If you already installed either of these packages, then you are all set! If you haven't, please see above for the installation instructions.

**Javascript context**
Stencila Desktop comes with the Javascript Execution Context enabled by default. It is implemented using the Node.js package.

However, if you want to use Javascript within a Stencila document hosted in some other application you'll need to install Stencila Node package.

* on **Linux** and **Mac OS**

```bash
npm install stencila-node
```

Then register the package's manifest so that it can be found by the Stencila Desktop and Stencila packages for other languages,

```bash
node -e "require('stencila-node').install()"
```

* on **Windows**
You need to first install `npm` following [these instructions](http://blog.teamtreehouse.com/install-node-js-npm-windows).
Open the Windows terminal (go to Start and then in "Search for programmes and files" type `cmd`). Then type in:

```bash
npm install stencila-node
```

Then register the package's manifest so that it can be found by the Stencila Desktop and Stencila packages for other languages,

```bash
node -e "require('stencila-node').install()"
```


## Stencila Docker image

If you have [Docker](https://docs.docker.com/install/) installed, you can use our [stencila/alpha](https://hub.docker.com/r/stencila/alpha/) Docker image. The image
comes with execution contexts for Python, SQL, R and Javascript.

```sh
docker run -p 2100:2000 stencila/alpha
```
