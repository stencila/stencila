import { LitElement } from "lit";
import { property } from "lit/decorators";

/**
 * Abstract base class for theme-able document views
 */
export abstract class ThemedElement extends LitElement {
  /**
   * The theme to apply to the document view
   */
  @property()
  theme: string = "default";

  /**
   * Set the theme of the document
   *
   * This function adds the theme stylesheet to the <head> if it does not
   * yet exist and removes the stylesheet for the current theme.
   *
   * This takes the approach of creating a new `<link title=theme:<THEME> rel=stylesheet>`
   * element in the <head> of the document (if it does not exist). This allows us to take
   * advantage of the browser's builtin caching (instead of say `fetch()`ing the CSS file).
   */
  setTheme(theme: string) {
    const adoptTheme = () => {
      // Find the stylesheet for the theme
      let styleSheet: CSSStyleSheet;
      for (const sheet of document.styleSheets) {
        if (sheet.title === `theme:${theme}`) {
          styleSheet = sheet;
          break;
        }
      }
      if (!styleSheet) {
        throw new Error(`Stylesheet for theme "${theme}" not loaded yet!`);
      }

      // It is only possible to adopt constructed stylesheets so
      // we have to serialize the parse rules in the new stylesheet
      // into CSS and construct a new stylesheet using that.
      const newCSS = [...styleSheet.cssRules]
        .map((rule) => rule.cssText)
        .join("");
      const constructedStyleSheet = new CSSStyleSheet();
      constructedStyleSheet.replaceSync(newCSS);
      this.shadowRoot.adoptedStyleSheets = [constructedStyleSheet];
    };

    // Check if the theme is already been loaded and if so just adopt it
    const alreadyLoaded =
      document.head.querySelector(`link[title="theme:${theme}"]`) !== null;
    if (alreadyLoaded) {
      adoptTheme();
      return;
    }

    // Create a <link> element for the theme and when it is loaded adopt it
    const newStyleSheet = document.createElement("link") as HTMLLinkElement;
    newStyleSheet.title = `theme:${theme}`;
    newStyleSheet.rel = "stylesheet";
    newStyleSheet.type = "text/css";
    newStyleSheet.href = `/~static/dev/themes/${theme}.css`;
    newStyleSheet.onload = () => adoptTheme();
    document.head.appendChild(newStyleSheet);
  }

  /**
   * Override `LitElement.update` so that if there has been a change
   * of theme it can be adopted by this element's Shadow DOM.
   */
  override update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties);

    if (changedProperties.has("theme")) {
      this.setTheme(this.theme);
    }
  }
}
