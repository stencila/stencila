import {
  DefaultDOMElement, Component,
  platform,
} from 'substance'
import getBoundingRect from '../util/getBoundingRect'

export default class SheetScrollbar extends Component {

  didMount() {
    this._updatePositions()
    this.props.viewport.on('scroll', this._onScroll, this)
  }

  dispose() {
    this.props.viewport.off(this)
  }

  didUpdate() {
    this._updatePositions()
  }

  render($$) {
    const horizontal = this._isHorizontal()
    let el = $$('div')
      .addClass('sc-sheet-scrollbar')
      .addClass(horizontal ? 'sm-horizontal' : 'sm-vertical')

    el.append(
      $$('div').addClass('se-lspace'),
      this._renderScrollArea($$),
      this._renderButtons($$),
      $$('div').addClass('se-rspace')
    )
    return el
  }

  _renderScrollArea($$) {
    let scrollArea = $$('div').ref('scrollArea').addClass('se-scroll-area')
    let thumb = $$('div').ref('thumb').addClass('se-thumb')
      .on('mousedown', this._onMousedownThumb)
    scrollArea.append(thumb)
    scrollArea.on('mousedown', this._onMousedownScrollArea)
    return scrollArea
  }

  _renderButtons($$) {
    const iconProvider = this.context.iconProvider
    const horizontal = this._isHorizontal()
    let buttons = $$('div').addClass('se-buttons')
    let prev = $$('button').ref('prev').addClass('se-prev').addClass('se-button')
      .on('mousedown', this._onMousedownPrev)
    let next = $$('button').ref('next').addClass('se-next').addClass('se-button')
      .on('mousedown', this._onMousedownNext)
    if (horizontal) {
      prev.append(iconProvider.renderIcon($$, 'sheet-scroll-left'))
      next.append(iconProvider.renderIcon($$, 'sheet-scroll-right'))
    } else {
      prev.append(iconProvider.renderIcon($$, 'sheet-scroll-up'))
      next.append(iconProvider.renderIcon($$, 'sheet-scroll-down'))
    }
    buttons.append(prev, next)
    return buttons
  }

  _isHorizontal() {
    return this.props.axis === 'x'
  }

  _updatePositions() {
    const sheet = this.props.sheet
    const viewport = this.props.viewport
    const horizontal = this._isHorizontal()
    let factor, scrollFactor, scrollbarSize
    if (horizontal) {
      factor = (viewport.endCol-viewport.startCol+1)/sheet.getColumnCount()
      scrollFactor = viewport.startCol/sheet.getColumnCount()
      scrollbarSize = this.refs.scrollArea.el.getWidth()
    } else {
      factor = (viewport.endRow-viewport.startRow+1)/sheet.getRowCount()
      scrollFactor = viewport.startRow/sheet.getRowCount()
      scrollbarSize = this.refs.scrollArea.el.getHeight()
    }
    let thumbSize = factor * scrollbarSize
    let pos = scrollFactor * scrollbarSize
    if (horizontal) {
      this.refs.thumb.css({
        left: pos,
        width: thumbSize
      })
    } else {
      this.refs.thumb.css({
        top: pos,
        height: thumbSize
      })
    }
  }

  _onResize() {
    // do a full rerender when window gets resized
    this.rerender()
  }

  _onMousedownThumb(e) {
    e.stopPropagation()
    e.preventDefault()
    // console.log('_onMouseDownThumb', e)
    if (platform.inBrowser) {
      // temporarily, we bind to events on window level
      // because could leave the this element's area while dragging
      let _window = DefaultDOMElement.wrap(window)
      _window.on('mousemove', this._onMoveThumb, this)
      _window.on('mouseup', this._onMouseUp, this)
    }
  }

  _onMousedownScrollArea(e) {
    // same as when mousedowning in the thumb
    this._onMousedownThumb(e)
    // plus moving the thumb to the start position
    this._onMoveThumb(e)
  }

  _onMousedownPrev(e) {
    e.stopPropagation()
    e.preventDefault()
    const viewport = this.props.viewport
    if (this._isHorizontal()) {
      viewport.shift(0, -1)
    } else {
      viewport.shift(-1, 0)
    }
  }

  _onMousedownNext(e) {
    e.stopPropagation()
    e.preventDefault()
    const viewport = this.props.viewport
    if (this._isHorizontal()) {
      viewport.shift(0, 1)
    } else {
      viewport.shift(1, 0)
    }
  }

  _onMouseUp(e) {
    e.stopPropagation()
    e.preventDefault()
    this._relax()
  }

  _onMoveThumb(e) {
    e.stopPropagation()
    e.preventDefault()
    const viewport = this.props.viewport
    const rect = getBoundingRect(this.refs.scrollArea.el)
    // TODO: we should consider at which position the user started
    // dragging the thumb instead of always using 0.5
    if (this._isHorizontal()) {
      let thumbSize = this.refs.thumb.el.getWidth()
      let clientPos = e.clientX - 0.5*thumbSize
      let size = rect.width
      let pos = Math.max(0, Math.min(size, clientPos - rect.left))
      let factor = pos / size
      let newCol = Math.floor(factor*viewport.M)
      viewport.shift(0, newCol-viewport.startCol)
    } else {
      let thumbSize = this.refs.thumb.el.getHeight()
      let clientPos = e.clientY - 0.5*thumbSize
      let size = rect.height
      let pos = Math.max(0, Math.min(size, clientPos - rect.top))
      let factor = pos / size
      let newRow = Math.floor(factor*viewport.N)
      viewport.shift(newRow-viewport.startRow, 0)
    }
  }

  _relax() {
    if (platform.inBrowser) {
      let _window = DefaultDOMElement.wrap(window)
      _window.off(this)
    }
  }

  _onScroll() {
    this._updatePositions()
  }

}
