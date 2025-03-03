# Stencila Ghost Theme

## Introduction

This package is a custom theme for the Stencila website hosted on [Ghost](https://ghost.org/). It features a setup that allows viewing [posts](https://ghost.org/help/posts/) in a technical document style page or in an article style. Additionally, it provides Stencila web components for embedding interactive content for posts and pages that.

## Features

- Technical document and article-style layouts
- Stencila web components integration
- Customizable routing using Ghost's [custom routing](https://ghost.org/docs/themes/routing/)
- TailwindCSS for styling
- Configurable navigation and social links

## Installation

1. Navigate to the ghost theme directory in the stencila repository:
    ```sh
    cd ./ghost
    ```
2. Install dependencies:
    ```sh
    npm install
    ```
3. Compile stencila web components:
    ```sh
    make compile
    ```

## Local Development

This theme was built off the [starter theme repository](https://github.com/TryGhost/Starter) provided by ghost.

To develop this theme locally, follow the installation steps then complete 

### Setup

1. Install the [ghost-cli](https://ghost.org/docs/ghost-cli/) package globally

    ```
    npm install -g ghost-cli@latest
    ```

2. Install Ghost in a local directory following this [guide](https://ghost.org/docs/ghost-cli/).
  
3. Create a sym-link between the theme directory and the local ghost `content/themes` directory:

   ```sh
   ln -s /path/to/stencila/repo/ghost /ghost/content/themes/stencila
   ```

4. In the browser navigate to the ghost admin interface (usually `localhost:2368/ghost`).
5. Activate the theme in **Settings** > **Design** > **Installed**.
6. Upload `stencila-site-routes.yml` in **Labs** > **Routes**.
7. Start the development server in the theme directory:

   ```sh
   npm run dev
   ```

### Templates

Ghost use the [handlebars](https://handlebarsjs.com/) library for page [templating](https://ghost.org/docs/themes/structure/#templates).

### Stylesheets

This theme uses [TailwindCSS](https://tailwindcss.com/) along with some default Ghost styles (`assets/css/ghost`). Stencila styles are compiled with web components and included via a link in the <head>:
```html
<link rel="stylesheet" href="{{asset 'built/stencila-web/ghost.css'}}">
```

Dynamic styles for raw/styled Stencila blocks are compiled at runtime using [Twind](https://twind.dev/).


## Routing Setup

The stencila site requires certain routes, use the ghost theme [dynamic routing](https://ghost.org/docs/themes/routing/) feature.

First configure the routes in your ghost instance:
1. Go to your Ghost admin panel.
2. Navigate to **Settings** > **Labs**.
3. In the 'Routes' field, click "Upload routes file" to upload the `stencila-site-routes.yml` file from this repository.

*If using building an alternate site with this theme from scratch there is a `base-theme-routes.yaml` file which contains minimal*

## Setting Up Pages and Posts

### Home Page

To create a new ghost page to act as the sites landing page

1. In the admin panel, create a new page (or use an existing one).
2. In the page editing view, open the settings sidebar.
3. Add `home` into the slug field and save the page.

### Doc Viewer Pages

Currently the stencila site has a document page for the **Documentation** and **Schema reference**

#### Page Setup

1. Create a new page.
2. In the page editing view, open the sidebar.
3. For the Documentation page add `docs` to the slug input, for the **Schema reference** add the `schema` slug.
4. Add the internal tag `#doc-page` to both pages. 
5. To create menu sections (e.g., 'Get Started' or 'API Reference'), add these as tags in the page's tag field.
6. The content on this page will act as the document's cover/welcome page.


### Publishing posts

#### Adding Posts to the Doc View Page

1. Create a new post.
2. Add the internal `#doc` tag, if a schema reference post also add the `#schema`.
3. Add the appropriate section tag (e.g., 'Get Started').
4. Posts are sorted by publication date, so adjust accordingly to change the order.

#### Posts for News and Changelog

Adding posts to the 'Changelog' or 'News' collections, just tag the post with 'Changelog' and 'News' tags reqpectively.


If another collection is needed, you can add a collection the routes.yaml file, or use the default [taxonmomy routing](https://ghost.org/docs/themes/routing/#taxonomies).


### Reference tab;e

This table has the pages as layed out in the `stencila` and the relevant template and any tags or slugs required for the page.

| Name       | Template             | Slug           | Tags       | 
|------------|----------------------|----------------|------------|
| Home       | `home.hbs`           | `home`         |            |
| Documentation/Schema | `index.hbs` | `docs`/`schema` | `#doc-page` |
| Changelog*  | `changelog.hbs`      |                |            |
| News*       | `tag.hbs`            |                |            |

*\*does not require a page to be created in admin interface*


## Published Stencila Posts/Pages

If a stencila document is published to ghost using the `stencila publish ghost`

Stencila docs published as pages/posts will support Stencila web components. If a post is tagged with `#stencila`, the Stencila UI for nodes will be available.

To have cards expanded by default, you can also add the `#stencila-expand` to the post tags.


## Other Configuration options

### Social Links

Social links appear in the footer. Ghost supports Facebook and Twitter; additional supported links include:
- WhatsApp
- Instagram
- GitHub
- Discord
- YouTube

To configure, go to **Settings** > **Design & Branding** > **Theme** and enter your social media links.

### Navigation

Ghost provides primary and secondary navigation:
- **Primary navigation**: Site header menu.
- **Secondary navigation**: Right-side footer or header buttons.

Pages tagged with `#doc-page` automatically appear under the "Resources" menu in the header, so do not need to be added to the navigation.


## Tags

### Internal Tags

- `#doc`: Marks a post as a documentation entry.
- `#schema`: Used with `#doc` for schema-related posts.
- `#doc-page`: Tags pages that display documentation.
- `#stencila`: Enables Stencila UI for pages and posts.
- `#stencila-expand`: Enables expandable Stencila UI.

### General Tags

- **Changelog**: Adds a post to the 'Changelog' collection.
- **News**: Adds a post to the 'News' collection.

## Custom Config Properties

Values can be edited in the admin interface under **Design & Branding** > **Theme**:
- `discord`: Discord server link.
- `github`: GitHub repository link.
- `whatsapp`: WhatsApp link.
- `instagram`: Instagram page link.
- `youtube`: YouTube channel link.
- `copyright`: Company name and year, will be inserted in the footer with &copy;.

