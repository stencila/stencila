![Stencila](http://static.stenci.la/logo-name-400x88.png)

# Browser module

This is Stencila's the browser module. 
It provides an interface to Stencila component from web browsers such as Google Chrome, Mozilla Firefox and Microsoft Internet Explorer.
These are developers notes for the browser module.

## Dependencies

We use (`bower`)[http://twitter.github.com/bower/] for package management.
It really is very useful and let us ditch our own tedious download-unzip-rename Makefile tasks!
There is a task in the Makefile for getting bower to download all the required packages:

```sh
make requirements
```

### Javascript inheritance

We use (Ken Power's version)[https://github.com/KenPowers/Base.js-Module] of Dean Edwards' venerable (`Base.js`)[http://dean.edwards.name/weblog/2006/03/base/].
Base.js provides for simple, succinct class definitions without having to refer
to `prototype` all the time and without the size and complexity of some other object oriented Javascript frameworks.

### Javascript HTML templates

We use the most excellent (`transparency`)[https://github.com/leonidas/transparency] for templating.
It saves the duplication of writing class and/or id attributes that are the same as the template tags.
In fact, its a similar approach to templateing used by Stencila's own stencils and when we get around to a Javascript port of those we might switch to using them!

### Styling

We use (`less`)[http://lesscss.org/] so we can write, aaahhh less, CSS.

### Cross-browser compatability

We use (`html5shiv`)(https://github.com/aFarkas/html5shiv) to add support for HTML5 elements to Internet Explorer 6-9, Safari 4.x (and iPhone 3.x), and Firefox 3.x.
Html5shiv makes these older bowsers recognise HTML5 elements like `<article>` so that styling can be applied to them without resorting to Javascript.
This script has to go into the `<head>` of the page so can not be loaded by `stencila-boot.js`.

We use (`normalize.css`)[http://nicolasgallagher.com/about-normalize-css/] to acheive cross-browser consistency in styling.
Normalise.css attempts to normalise styles applied to HTML elements while still preserving browser defauls.
As such it represents an alternative to (CSS reset approaches)[http://meyerweb.com/eric/tools/css/reset/]

We use (`ie7-js`)[http://code.google.com/p/ie7-js/] to make Microsoft Internet Explorer (IE) "behave like a standards-compilant browser".
Including ie7-js gives IE versions 5.5 to 8 missing functionality so that they behave like modern browsers.

We also use Paul Irish's (conditional IE CSS classes)[http://paulirish.com/2008/conditional-stylesheets-vs-css-hacks-answer-neither/]
to detect which version of IE is being used and (FUOC avoidance method)[http://paulirish.com/2009/avoiding-the-fouc-v3/]. 
At present the IE CSS classes are only used by `stencila-boot.js` to determine whether or not to load `ie7-js`.

We use (`modernizr`)[http://modernizr.com/] for HTML and CSS feature detection.
Modernizer allows for (polyfills)[https://github.com/Modernizr/Modernizr/wiki/HTML5-Cross-browser-Polyfills] to be loaded if a browser does not have a particular feature.

## Building

Currently there is a Makefile that does file concatenation and uses
`uglify` to minimise Javascript:

```sh
make all
```
In the future we might more to using
something like (`grunt.js`)[http://gruntjs.com/]

## Testing

We are using (`Jasmine`)[http://pivotal.github.com/jasmine/] as a testing framework.

```sh
bower install jasmine
```

For reporting of JUnit XML  files also install `jasmine-reporters`

```sh
bower install git://github.com/larrymyers/jasmine-reporters.git
```

To run the tests in a browser start the Stencila server locally and
navigate to (http://localhost:55555/tests/run.html)[http://localhost:55555/tests/run.html].


