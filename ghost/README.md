# Stencila ghost theme

## Routing

This them will utililse ghosts [custom routing](https://ghost.org/docs/themes/routing/).

TO setup this theme in a ghost instance. Go to your Ghost admin navigate to **Settings** > **Labs**, then in the 'Routes' section click "Upload routes file" to upload the `base-theme-routes.yml` file in this directory

## Setting up pages

### home page

To create a home page, In the admin panel, create a new page (or use an existing one).

In the page editing view, open the settings sidebar and add 'home' into the slug field. and save the page.

### doc view page

#### Page setup

To create a doc viewer page, create a new page, In the page editing view, open the sidebar and add 'docview' to the slug input.

To create the menu sections for your docs (for example: 'Get Started' or 'API Reference'), add these items as tags to the page's tag field.

The content on the page will act as the document's cover/welcome page, will appear at the top of the sidebar menu.

#### Creating posts for the doc view page

To add posts to the doc view page, create a new post and add the internal `#doc` tag, As well as the tag for the section of the docviewer it will appear under. eg. for a post called 'Introduction' to appear under the 'Get Started' section, you would add the tags '#doc' and 'Get Started' to the posts tags

By default ghost will sort posts in order of publishing date, so the current method for changing the order the post show in the menu would be to alter the posts publication date.


### Creating additional pages for articles, blog posts, tutorials, changelogs etc...

To create posts that are not and a display page, ordered by publication, for example a blog, you can utilise ghost's tag [taxonomy](https://ghost.org/docs/themes/routing/#taxonomies) routing.

For example, If you want to link users to apage that showing all posts tagged with 'article'.

Go the to admin panel **Settings** -> **Navigation** and click **'customise'**. In the primary navigation tab add a new entry, and in the route, add "http://{{your site domian}}/tag/{{tag slug}}", save and then refresh your site to see this route in the primary navigation. 

*note, the slug in the page route, must match the tag's 'slug' field you want to use. The slug may be different from the tag name, to find the slug; navigate to the 'Tags' on the main ghost admin screen, and check the 'slug' column.

Add a new navigation route to 

### create multiple docview pages

This will require altering the base `theme-routes.yml` file and uploading into the admin settings again.

Implementing this theme for the stencila website, we have two doc viewer style pages. We have replaced the base 'docviewer' collection with two document pages 'docs' and 'schema'.

```yml
collections:
  /main-docs/:
    permalink: /docs/{slug}/
    template: index
    data: page.docs
    filter: "tag:hash-doc+tag:-hash-schema"

  /schema-docs/:
    permalink: /schema/{slug}/
    template: index
    data: page.schema
    filter: "tag:hash-doc+tag:hash-schema"
```

To implement this or a similar structure, upload the routes.yml file via the admin settings "Labs" sections. We will create two pages, one with the slug 'docs' and another with the slug 'schema'.

All posts are still tagged with the #doc tag, and the schema docs are also tagged with the '#schema' tag.

__If there is a crossover of sections on either of the pages (eg. both pages have the 'API Reference' tag). You will need to create seperate tags for each page, or the #doc will show up under both sections on both pages. To achieve this you can name the seperate tags 'API Reference - docs' & 'API Reference - schema' then you can edit the tags 'Meta title' field so they both display as 'API Reference'.__



## Misc.

### Configuring social links

All social links currently shown in the page footer.

Ghost provides support for facebook and x(formaly twitter) which can be configured in the admin menu.

This theme adds support for 
'whatsapp', 'instagram', 'github', 'discord', 'youtube'.

To add these links, in the admin view go to **Settings** -> **Design & branding**, click "customise" to open the design view, then in the sidebar click "theme", here you will see the inputs for various social media links. Fill these out and they will appear in the page footer.

### configuring site navigation


Ghost sites have configurable primary and secondary navigation fields.

In the stencila theme, the primary navigation relates to the site header nav menu.

The secondary navigation is shown on the right side of the site footer.


## Local Development

This theme has created using the ghost theme [starter](https://github.com/TryGhost/Starter).

Install the theme's dependencies inside the stencila theme directory with `npm install`.

To start local development, [install ghost locally](https://ghost.org/docs/ghost-cli/) on your machine.
Then create a link to the theme in your local ghost's `content/themes` directory

```bash
ln -s /path/to/stencila-theme /ghost/content/themes/stencila
```

In the browser navigate to your local ghost admin site (usually http://localhost:2368/ghost/).
From the settings menu select **Theme** -> **Installed**, and activate 'stencila-ghost-theme'.

After initialising the theme, upload the relevant `routes.yml` file in **Labs** -> **Routes**, and setup the pages accordingly.

To enable development with hot-loading, In the stencila theme directory and `npm run dev` to start the dev server.

ghost themes use handlebars templating language, for more information refer to ghost's [developer documentaion](https://ghost.org/docs/themes/).



# ----------------------- 
# OLD README BELOW 


# Ghost Starter Theme

A starter framework for Ghost themes! Click **Use this template** to create a copy of this repo for everything you need to get started developing a custom Ghost theme.

&nbsp;

## First time using a Ghost theme?

Ghost uses a simple templating language called [Handlebars](http://handlebarsjs.com/) for its themes.

We've documented this starter theme pretty heavily so that it should be possible to work out what's going on just by reading the code and the comments. We also have a robust set of resources to help you build awesome custom themes:

- The official [theme documentation](https://ghost.org/docs/themes) is the complete resource for everything you need to know about Ghost theme development
- [Tutorials](https://ghost.org/tutorials/) offer a step-by-step guide to building the most common features in Ghost themes
- The [Ghost VS Code extension](https://marketplace.visualstudio.com/items?itemName=TryGhost.ghost) speeds up theme development and provides quick access to helpful info
- All of Ghost's official themes are [open source](https://github.com/tryghost) and are a great reference for learning how to create a theme

&nbsp;

## Starter theme features

🔁&nbsp;Livereload by default. See changes instantly in the browser whenever you save a file.

🔎&nbsp;Optimized for VS Code. Find the files you're looking for more easily.

🗃️&nbsp;Write modern JavaScript. Use ESM out of the box to write more manageable Javascript.

🗜️&nbsp;Assets optimized automatically. JavaScript and CSS are minified and transpiled by default.

👟&nbsp;Fast compile times, powered by [Rollup](https://rollupjs.org).

🦋&nbsp;Write next-gen CSS for today's browsers with [PostCSS](https://postcss.org/). Add the CSS tools you love via [`rollup.config.js`](rollup.config.js).

🚢&nbsp;Ghost's [GH Deploy Action](.github/workflows/deploy-theme.yml) included by default. [Learn more how to deploy your theme automatically](https://github.com/TryGhost/action-deploy-theme)

➕&nbsp;Extensible by design. Rollup's configuration structure makes it easy to add [any number of plugins easily](https://github.com/rollup/plugins). 

&nbsp;

## Theme structure

The main files are:

- [`default.hbs`](default.hbs) - The main template file
- [`index.hbs`](index.hbs) - Used for the home page
- [`post.hbs`](post.hbs) - Used for individual posts
- [`page.hbs`](page.hbs) - Used for individual pages
- [`tag.hbs`](tag.hbs) - Used for tag archives
- [`author.hbs`](author.hbs) - Used for author archives

One neat trick is that you can also create custom one-off templates just by adding the slug of a page to a template file. For example:

- `page-about.hbs` - Custom template for the `/about/` page
- `tag-news.hbs` - Custom template for `/tag/news/` archive
- `author-jamie.hbs` - Custom template for `/author/jamie/` archive

&nbsp;

## Development guide

The Starter theme provides a first-class development experience out of the box. 

&nbsp;

### Setup

To see realtime changes during development, symlink the Starter theme folder to the `content/themes` folder in your local Ghost install. 

```bash
ln -s /path/to/starter /ghost/content/themes/starter
```

Restart Ghost and select the Starter theme from **Settings**.

From the theme's root directory, install the dependencies:

```bash
npm install
```

If Node isn't installed, follow the [official Node installation guide](https://nodejs.org/).

&nbsp;

### Start development mode

From the Starter theme folder, start development mode:

```bash
npm run dev
```

Changes you make to your styles, scripts, and Handlebars files will show up automatically in the browser. CSS and Javascript will be compiled and output to the `built` folder.

Press `ctrl + c` in the terminal to exit development mode.

&nbsp;

### Build, zip, and test your theme

Compile your CSS and JavaScript assets for production with the following command:

```bash
npm run build
```

Create a zip archive:

```bash
npm run zip
```

Use `gscan` to test your theme for compatibility with Ghost:

```bash
npm run test
```

&nbsp;



## Copyright & License

Copyright (c) 2013-2025 Ghost Foundation - Released under the [MIT license](LICENSE).

