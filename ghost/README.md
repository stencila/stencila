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

To create a doc viewer page, create a new page, In the page editing view, open the sidebar and add 'docview' to the slug input, and add the internal tag #doc-page.

To create the menu sections for your docs (for example: 'Get Started' or 'API Reference'), add these items as tags to the page's tag field.

The content on the page will act as the document's cover/welcome page, will appear at the top of the sidebar menu.

#### Creating posts for the doc view page

To add posts to the doc view page, create a new post and add the internal `#doc` tag, As well as the tag for the section of the docviewer it will appear under. eg. for a post called 'Introduction' to appear under the 'Get Started' section, you would add the tags '#doc' and 'Get Started' to the posts tags

By default ghost will sort posts in order of publishing date, so the current method for changing the order the post show in the menu would be to alter the posts publication date.


### Creating additional pages for articles, blog posts, tutorials, changelogs etc...

To create posts that are not and a display page, ordered by publication, for example a blog, you can utilise ghost's tag [taxonomy](https://ghost.org/docs/themes/routing/#taxonomies) routing.

For example, If you want to link users to a page that showing all posts tagged with 'article'.

Go the to admin panel **Settings** -> **Navigation** and click **'customise'**. In the primary navigation tab add a new entry, and in the route, add "http://{{your site domian}}/tag/{{tag slug}}", save and then refresh your site to see this route in the primary navigation. 

Or if you want to define the routes porperly add collection to the routes.yaml, the example below is from stencila-site-routes.yml, and it clearly defines routes for 'changelog' and 'news' posts:

```yaml
# routes.yaml
collections: 
  /changelog/:
    permalink: /changelog/{slug}/
    template: tag
    data: tag.changelog
    filter: tag:changelog

  /news/:
    permalink: /news/{slug}/
    template: tag ## these are still using the base template for tags, but you can creat and use custom templates
    data: tag.news
    filter: tag:news

```

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
ln -s /path/to/stencila/repo/ghost /ghost/content/themes/stencila
```

Then compile the stencila web components into the theme by running the following `make` command from the stencila repo root:

```bash
make -C ghost compile
```

In the browser navigate to your local ghost admin site (usually http://localhost:2368/ghost/).
From the settings menu select **Theme** -> **Installed**, and activate 'stencila-ghost-theme'.

After initialising the theme, upload the relevant `routes.yml` file in **Labs** -> **Routes**, and setup the pages accordingly.

To enable development with hot-loading, In the stencila theme directory and `npm run dev` to start the dev server.

ghost themes use handlebars templating language, for more information refer to ghost's [developer documentaion](https://ghost.org/docs/themes/).

