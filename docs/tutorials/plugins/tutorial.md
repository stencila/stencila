# Stencila Plugins

> DRAFT

Plugins extend the functionality of Stencila to integrate additional capabilities like kernels and assistants. You can create new plugins using common frameworks and languages like Node and Python that will interoperate with Stencila.

This tutorial guides you through the development and testing of a new Stencila plugin. Before starting work on plugins, first [establish a Stencila development environment](https://github.com/stencila/stencila/blob/main/docs/tutorials/development-environments/tutorial.md).

## Initialize Your Plugin

To create a new plugin, start by creating the necessary default folders and files with a unique name and location to add to Stencila's plugin registry to make the new plugin available. Cloning and modifying an existing Stencila example plugin is the easiest way to make sure your plugin has the basic necessities.

### Example Plugins

1. [Node](https://github.com/stencila/plugin-node-template)
1. [Python](https://github.com/stencila/plugin-python-template)

## Customize Your Plugin

1. Clone an example plugin above to a directory/repository where your new plugin will live. Note that plugins live in their own, independent space and are registered in but not part of [the core Stencila codebase](https://github.com/stencila/stencila).
1. Pick an appropriate name for your plugin and make changes to all necessary name and location values in your cloned example.

## Register Your Plugin

For your plugin to be available in Stencila, it must be registered and linked.

1. Register your plugin locally in your copy of Stencila's core plugin registery file `[plugins.toml](https://github.com/stencila/stencila/blob/main/plugins.toml)` using the format `your-plugin-name = "URL for your plugin repository"`.
1. [Link](https://github.com/stencila/stencila/blob/main/docs/reference/cli.md#stencila-plugins-link) your plugin locally: `stencila plugins link your-plugin-name`.

## Test Your Plugin

Test if your plugin is available in your Stencila development environment and check its structure and syntax.

1. [List](https://github.com/stencila/stencila/blob/main/docs/reference/cli.md#stencila-plugins-list) registered plugins: `stencila plugins list`.
1. [Check](https://github.com/stencila/stencila/blob/main/docs/reference/cli.md#stencila-plugins-list) your plugin: `stencila plugins check your-plugin-name`.

## Continue Plugin Development

Once you have a registered and working plugin based on a template you can move forward to customize and extend your plugin's capabilities.

## Contribute Your Plugin to Stencila

When your plugin development is complete, you can contribute your plugin to Stencila by submitting a pull request to the main code repository that adds your plugin registration to the core plugin registery file `[plugins.toml](https://github.com/stencila/stencila/blob/main/plugins.toml)`.
