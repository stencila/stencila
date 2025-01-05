import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { unsafeSVG } from 'lit/directives/unsafe-svg.js'

/**
 * The SVG files are bundled as raw strings by Parcel due to the
 * use of "@parcel/transformer-inline-string" in `.parcelrc`.
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

import activity from './activity.svg'
import admonition from './admonition.svg'
import anthropic from './anthropic.svg'
import archive from './archive.svg'
import array from './array.svg'
import arrowBarUp from './arrow-bar-up.svg'
import arrowClockwise from './arrow-clockwise.svg'
import arrowLeftSquare from './arrow-left-square.svg'
import arrowNarrowUp from './arrow-narrow-up.svg'
import arrowRampRight3 from './arrow-ramp-right-3.svg'
import arrowRepeat from './arrow-repeat.svg'
import arrowRightSquare from './arrow-right-square.svg'
import arrowRight from './arrow-right.svg'
import arrowUpCircleFill from './arrow-up-circle-fill.svg'
import arrowUpCircle from './arrow-up-circle.svg'
import arrowsCollapse from './arrows-collapse.svg'
import arrowsExpand from './arrows-expand.svg'
import asterisk from './asterisk.svg'
import at from './at.svg'
import ban from './ban.svg'
import bars from './bars.svg'
import bash from './bash.svg'
import boxArrowInLeft from './box-arrow-in-left.svg'
import boxArrowUpRight from './box-arrow-up-right.svg'
import box from './box.svg'
import bracesAsterisk from './braces-asterisk.svg'
import braces from './braces.svg'
import brush from './brush.svg'
import building from './building.svg'
import cameraVideo from './camera-video.svg'
import cardText from './card-text.svg'
import chatRightDots from './chat-right-dots.svg'
import chatRightText from './chat-right-text.svg'
import chatSquareText from './chat-square-text.svg'
import checkCircleFill from './check-circle-fill.svg'
import checkCircle from './check-circle.svg'
import check from './check.svg'
import chevronDown from './chevron-down.svg'
import chevronLeft from './chevron-left.svg'
import chevronRight from './chevron-right.svg'
import circle from './circle.svg'
import clock from './clock.svg'
import cloudflare from './cloudflare.svg'
import codeChunk from './code-chunk.svg'
import codeSlash from './code-slash.svg'
import code from './code.svg'
import compass from './compass.svg'
import coneStriped from './cone-striped.svg'
import crosshair from './crosshair.svg'
import currencyDollar from './currency-dollar.svg'
import cursor from './cursor.svg'
import dashCircle from './dash-circle.svg'
import elifClause from './elif-clause.svg'
import elseClause from './else-clause.svg'
import exclamationCircle from './exclamation-circle.svg'
import exclamationTriangle from './exclamation-triangle.svg'
import eyeSlash from './eye-slash.svg'
import eye from './eye.svg'
import fastForwardCircle from './fast-forward-circle.svg'
import feather from './feather.svg'
import filePlay from './file-play.svg'
import filePlus from './file-plus.svg'
import fileTypeRaw from './filetype-raw.svg'
import fullscreen from './fullscreen.svg'
import gear from './gear.svg'
import google from './google.svg'
import handThumbsDownFill from './hand-thumbs-down-fill.svg'
import handThumbsDown from './hand-thumbs-down.svg'
import handThumbsUpFill from './hand-thumbs-up-fill.svg'
import handThumbsUp from './hand-thumbs-up.svg'
import hash from './hash.svg'
import heading from './heading.svg'
import highlights from './highlights.svg'
import hr from './hr.svg'
import ifBlock from './if-block.svg'
import ifClause from './if-clause.svg'
import imageAlt from './image-alt.svg'
import image from './image.svg'
import infoCircle from './info-circle.svg'
import javascript from './javascript.svg'
import json from './json.svg'
import latex from './latex.svg'
import lightbulb from './lightbulb.svg'
import lightning from './lightning.svg'
import list from './list.svg'
import lock from './lock.svg'
import markdown from './markdown.svg'
import mathBlock from './math-block.svg'
import mermaid from './mermaid.svg'
import mistral from './mistral.svg'
import nodejs from './nodejs.svg'
import ollama from './ollama.svg'
import openai from './openai.svg'
import paperclip from './paperclip.svg'
import paragraph from './paragraph.svg'
import person from './person.svg'
import playCircle from './play-circle.svg'
import play from './play.svg'
import plusCircle from './plus-circle.svg'
import postage from './postage.svg'
import python from './python.svg'
import questionCircle from './question-circle.svg'
import quote from './quote.svg'
import r from './r.svg'
import repeat from './repeat.svg'
import replaceBlock from './replace-block.svg'
import robot from './robot.svg'
import rosetteCheck from './rosette-check.svg'
import rosetteFillCheck from './rosette-fill-check.svg'
import sandbox from './sandbox.svg'
import shieldCheck from './shield-check.svg'
import skipEnd from './skip-end.svg'
import skipStart from './skip-start.svg'
import slashCircle from './slash-circle.svg'
import sliders from './sliders.svg'
import speedometer from './speedometer.svg'
import square from './square.svg'
import starFill from './star-fill.svg'
import stencilaColor from './stencila-color.svg'
import stencila from './stencila.svg'
import stopwatch from './stopwatch.svg'
import table from './table.svg'
import terminal from './terminal.svg'
import tex from './tex.svg'
import thermometer from './thermometer.svg'
import toggleOff from './toggle-off.svg'
import volumeUp from './volume-up.svg'
import xCircle from './x-circle.svg'
import x from './x.svg'

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
  exclamationCircle,
  exclamationTriangle,
  eye,
  eyeSlash,
  fastForwardCircle,
  feather,
  filePlay,
  filePlus,
  fileTypeRaw,
  fullscreen,
  gear,
  google,
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
  quickjs: javascript,
  json,
  latex,
  lightbulb,
  lightning,
  list,
  lock,
  markdown,
  mathBlock,
  mermaid,
  mistral,
  nodejs,
  'node.js': nodejs,
  ollama,
  openai,
  paperclip,
  paragraph,
  person,
  play,
  playCircle,
  plusCircle,
  postage,
  python,
  python3: python,
  questionCircle,
  quote,
  r,
  repeat,
  replaceBlock,
  robot,
  rosetteCheck,
  rosetteFillCheck,
  sandbox,
  shieldCheck,
  skipEnd,
  skipStart,
  slashCircle,
  sliders,
  speedometer,
  square,
  starFill,
  stencila,
  stencilaColor,
  stopwatch,
  table,
  terminal,
  tex,
  thermometer,
  toggleOff,
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
