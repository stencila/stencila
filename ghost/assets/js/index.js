// JavaScript files are compiled and minified during the build process to the assets/built folder. See available scripts in the package.json file.

// Import CSS
import "../css/index.css";

// Import JS
import mobileMenu from "./mobileMenu";
import infiniteScroll from "./infiniteScroll";
import { sidebar, menuCollapse } from './docviewer/sidebar';
import generateTableOfContents from './docviewer/toc';

import subnavDropdown from './subnavDropdown';

import pagination from './pagination';
import processRuntimeTwind from './twind';


// Call the menu and infinite scroll functions
mobileMenu();
infiniteScroll();
sidebar();
pagination();
menuCollapse();
processRuntimeTwind();
generateTableOfContents();
subnavDropdown();
