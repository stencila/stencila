import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

/**
 * The SVG files are imported as raw strings using Vite's `?raw` suffix.
 * As such, we need to use `unsafeSVG` below.
 *
 * An alternative would be to use Lit `svg` and have the SVGs actually
 * inlined into Typescript. But that could get unwieldily to manage.
 *
 * See `./icons-shoelace.sh` for a script that copies over Shoelace (Bootstrap icons).
 * See `./LICENSE` associated with those.
 *
 * Other, programming and development related icons from https://devicon.dev/.
 *
 * For proper scaling in VS Code, images should not have `width` and `height`
 * attributes but should have a `viewBox` (it may be being stripped e.g. https://github.com/svg/svgo/issues/1128#issuecomment-628208565)
 */

import activity from './activity.svg?raw'
import admonition from './admonition.svg?raw'
import anthropic from './anthropic.svg?raw'
import archive from './archive.svg?raw'
import array from './array.svg?raw'
import arrowBarUp from './arrow-bar-up.svg?raw'
import arrowClockwise from './arrow-clockwise.svg?raw'
import arrowLeftSquare from './arrow-left-square.svg?raw'
import arrowNarrowUp from './arrow-narrow-up.svg?raw'
import arrowRampRight3 from './arrow-ramp-right-3.svg?raw'
import arrowRepeat from './arrow-repeat.svg?raw'
import arrowRightSquare from './arrow-right-square.svg?raw'
import arrowRight from './arrow-right.svg?raw'
import arrowUpCircleFill from './arrow-up-circle-fill.svg?raw'
import arrowUpCircle from './arrow-up-circle.svg?raw'
import arrowsCollapse from './arrows-collapse.svg?raw'
import arrowsExpand from './arrows-expand.svg?raw'
import asterisk from './asterisk.svg?raw'
import at from './at.svg?raw'
import ban from './ban.svg?raw'
import bars from './bars.svg?raw'
import bash from './bash.svg?raw'
import boxArrowInLeft from './box-arrow-in-left.svg?raw'
import boxArrowUpRight from './box-arrow-up-right.svg?raw'
import box from './box.svg?raw'
import bracesAsterisk from './braces-asterisk.svg?raw'
import braces from './braces.svg?raw'
import brush from './brush.svg?raw'
import building from './building.svg?raw'
import cameraVideo from './camera-video.svg?raw'
import cardText from './card-text.svg?raw'
import chatRightDots from './chat-right-dots.svg?raw'
import chatRightText from './chat-right-text.svg?raw'
import chatSquareText from './chat-square-text.svg?raw'
import checkCircleFill from './check-circle-fill.svg?raw'
import checkCircle from './check-circle.svg?raw'
import check from './check.svg?raw'
import chevronDown from './chevron-down.svg?raw'
import chevronLeft from './chevron-left.svg?raw'
import chevronRight from './chevron-right.svg?raw'
import circle from './circle.svg?raw'
import clock from './clock.svg?raw'
import cloudflare from './cloudflare.svg?raw'
import codeChunk from './code-chunk.svg?raw'
import codeSlash from './code-slash.svg?raw'
import code from './code.svg?raw'
import compass from './compass.svg?raw'
import coneStriped from './cone-striped.svg?raw'
import crosshair from './crosshair.svg?raw'
import currencyDollar from './currency-dollar.svg?raw'
import cursor from './cursor.svg?raw'
import dashCircle from './dash-circle.svg?raw'
import elifClause from './elif-clause.svg?raw'
import elseClause from './else-clause.svg?raw'
import eslint from './eslint.svg?raw'
import exclamationCircle from './exclamation-circle.svg?raw'
import exclamationTriangle from './exclamation-triangle.svg?raw'
import externalLink from './external-link.svg?raw'
import eyeSlash from './eye-slash.svg?raw'
import eye from './eye.svg?raw'
import fastForwardCircle from './fast-forward-circle.svg?raw'
import feather from './feather.svg?raw'
import filePlay from './file-play.svg?raw'
import filePlus from './file-plus.svg?raw'
import files from './files.svg?raw'
import fileTypeRaw from './filetype-raw.svg?raw'
import fullscreen from './fullscreen.svg?raw'
import gear from './gear.svg?raw'
import google from './google.svg?raw'
import groq from './groq.svg?raw'
import handThumbsDownFill from './hand-thumbs-down-fill.svg?raw'
import handThumbsDown from './hand-thumbs-down.svg?raw'
import handThumbsUpFill from './hand-thumbs-up-fill.svg?raw'
import handThumbsUp from './hand-thumbs-up.svg?raw'
import hash from './hash.svg?raw'
import heading from './heading.svg?raw'
import highlights from './highlights.svg?raw'
import hr from './hr.svg?raw'
import ifBlock from './if-block.svg?raw'
import ifClause from './if-clause.svg?raw'
import imageAlt from './image-alt.svg?raw'
import image from './image.svg?raw'
import infoCircle from './info-circle.svg?raw'
import javascript from './javascript.svg?raw'
import json from './json.svg?raw'
import kuzu from './kuzu.svg?raw'
import latex from './latex.svg?raw'
import lightbulb from './lightbulb.svg?raw'
import lightningChargeFill from './lightning-charge-fill.svg?raw'
import lightning from './lightning.svg?raw'
import list from './list.svg?raw'
import lock from './lock.svg?raw'
import markdown from './markdown.svg?raw'
import mathBlock from './math-block.svg?raw'
import mermaid from './mermaid.svg?raw'
import mistral from './mistral.svg?raw'
import moon from './moon.svg?raw'
import nodejs from './nodejs.svg?raw'
import ollama from './ollama.svg?raw'
import openai from './openai.svg?raw'
import paperclip from './paperclip.svg?raw'
import paragraph from './paragraph.svg?raw'
import person from './person.svg?raw'
import playCircle from './play-circle.svg?raw'
import playFill from './play-fill.svg?raw'
import play from './play.svg?raw'
import plusCircle from './plus-circle.svg?raw'
import postage from './postage.svg?raw'
import prettier from './prettier.svg?raw'
import python from './python.svg?raw'
import questionCircle from './question-circle.svg?raw'
import quote from './quote.svg?raw'
import r from './r.svg?raw'
import repeat from './repeat.svg?raw'
import robot from './robot.svg?raw'
import rosetteCheck from './rosette-check.svg?raw'
import rosetteFillCheck from './rosette-fill-check.svg?raw'
import ruff from './ruff.svg?raw'
import sandbox from './sandbox.svg?raw'
import shieldCheck from './shield-check.svg?raw'
import skipEnd from './skip-end.svg?raw'
import skipStart from './skip-start.svg?raw'
import slashCircle from './slash-circle.svg?raw'
import sliders from './sliders.svg?raw'
import speedometer from './speedometer.svg?raw'
import squareFill from './square-fill.svg?raw'
import square from './square.svg?raw'
import starFill from './star-fill.svg?raw'
import stencilaColor from './stencila-color.svg?raw'
import stencila from './stencila.svg?raw'
import stopwatch from './stopwatch.svg?raw'
import sun from './sun.svg?raw'
import table from './table.svg?raw'
import terminal from './terminal.svg?raw'
import tex from './tex.svg?raw'
import thermometer from './thermometer.svg?raw'
import toggleOff from './toggle-off.svg?raw'
import trash from './trash.svg?raw'
import volumeUp from './volume-up.svg?raw'
import xCircle from './x-circle.svg?raw'
import x from './x.svg?raw'

const icons = {
  activity,
  admonition,
  anthropic,
  archive,
  array,
  arrowBarUp,
  arrowClockwise,
  arrowLeftSquare,
  arrowNarrowUp,
  arrowRampRight3,
  arrowRepeat,
  arrowRight,
  arrowRightSquare,
  arrowsCollapse,
  arrowsExpand,
  arrowUpCircle,
  arrowUpCircleFill,
  asterisk,
  at,
  ban,
  bars,
  bash,
  box,
  boxArrowInLeft,
  boxArrowUpRight,
  braces,
  bracesAsterisk,
  brush,
  building,
  cameraVideo,
  cardText,
  chatRightDots,
  chatRightText,
  chatSquareText,
  check,
  checkCircle,
  checkCircleFill,
  chevronDown,
  chevronLeft,
  chevronRight,
  circle,
  clock,
  cloudflare,
  code,
  codeChunk,
  codeSlash,
  compass,
  coneStriped,
  crosshair,
  cursor,
  currencyDollar,
  dashCircle,
  elifClause,
  elseClause,
  eslint,
  exclamationCircle,
  exclamationTriangle,
  externalLink,
  eye,
  eyeSlash,
  fastForwardCircle,
  feather,
  filePlay,
  filePlus,
  files,
  fileTypeRaw,
  fullscreen,
  gear,
  google,
  groq,
  handThumbsDown,
  handThumbsDownFill,
  handThumbsUp,
  handThumbsUpFill,
  hash,
  heading,
  highlights,
  hr,
  ifBlock,
  ifClause,
  image,
  imageAlt,
  infoCircle,
  javascript,
  kuzu,
  quickjs: javascript,
  json,
  latex,
  lightbulb,
  lightningChargeFill,
  lightning,
  list,
  lock,
  markdown,
  mathBlock,
  mermaid,
  mistral,
  moon,
  nodejs,
  'node.js': nodejs,
  ollama,
  openai,
  paperclip,
  paragraph,
  person,
  play,
  playCircle,
  playFill,
  plusCircle,
  postage,
  prettier,
  python,
  python3: python,
  questionCircle,
  quote,
  r,
  repeat,
  robot,
  rosetteCheck,
  rosetteFillCheck,
  ruff,
  sandbox,
  shieldCheck,
  skipEnd,
  skipStart,
  slashCircle,
  sliders,
  speedometer,
  square,
  squareFill,
  starFill,
  stencila,
  stencilaColor,
  stopwatch,
  sun,
  table,
  terminal,
  tex,
  thermometer,
  toggleOff,
  trash,
  volumeUp,
  xCircle,
  x,
}

export type IconName = keyof typeof icons

/**
 * Get an icon name from a string or else return null
 */
export function iconMaybe(name: string): IconName | null {
  return name in icons ? (name as IconName) : null
}

/**
 * An icon used in the UI
 *
 * Previously we used Shoelace's <sl-icon> but that caused
 * issues with builds and distribution (because it does not allow
 * SVGs to be bundled; they need to be separate files). This
 * approach also has the advantages of not having to copy around
 * static folders and strong typing the icon name.
 */
@customElement('stencila-ui-icon')
export abstract class UIIcon extends LitElement {
  @property()
  name: IconName

  constructor(name: IconName) {
    super()
    this.name = name
  }

  static override styles = css`
    :host {
      display: block;
      width: 1em;
      height: 1em;
      box-sizing: content-box !important;
    }

    svg {
      display: block;
      height: 100%;
      width: 100%;
    }
  `

  override render() {
    return html`${unsafeSVG(icons[this.name])}`
  }
}
