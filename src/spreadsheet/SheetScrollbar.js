import {
  DefaultDOMElement, Component,
  platform,
} from 'substance'
import getBoundingRect from '../util/getBoundingRect'

export default class SheetScrollbar extends Component {

  didMount() {
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    this._updatePositions()
    this.props.viewport.on('scroll', this._onScroll, this)
  }

  dispose() {
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
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
    if (factor < 1) {
      this.el.removeClass('sm-hidden')
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
    } else {
      this.el.addClass('sm-hidden')
    }
  }

  _onResize() {
    // do a full rerender when window gets resized
    this.rerender()
  }

  _onMousedownThumb(e) {
    e.stopPropagation()
    e.preventDefault()
    console.log('_onMouseDownThumb', e)
  }

  _onMousedownScrollArea(e) {
    e.stopPropagation()
    e.preventDefault()
    console.log('_onMousedownScrollArea', e)
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

  _onMouseDown(e) {
    e.stopPropagation()
    e.preventDefault()
    this._mouseDown = true

    if (e.target === this.refs.prev.getNativeElement()) {
      console.log('PREV')
    } else if (e.target === this.refs.next.getNativeElement()) {
      console.log('NEXT')
    } else {
      if (platform.inBrowser) {
        // temporarily, we bind to events on window level
        // because could leave the this element's area while dragging
        let _window = DefaultDOMElement.wrap(window)
        _window.on('mousemove', this._onMouseMove, this)
        _window.on('mouseup', this._onMouseUp, this)
      }
      // TODO: if clicked outside of the thumb
      // directly scroll to the position
      if (e.target !== this.refs.thumb.getNativeElement()) {
        const sheet = this.props.sheet
        const viewport = this.props.viewport
        const horizontal = this._isHorizontal()
        let rect = getBoundingRect(this.el)
        if (horizontal) {
          let x = e.clientX - rect.left
          let factor = x / rect.width
          let newStartCol = factor * sheet.getColumnCount()
          if (viewport.startCol !== newStartCol) {
            viewport.shift(0, newStartCol-viewport.startCol)
          }
        } else {
          let y = e.clientY - rect.top
          let factor = y / rect.height
          let newStartRow = factor * sheet.getRowCount()
          if (viewport.startRow !== newStartRow) {
            viewport.shift(newStartRow-viewport.startRow, 0)
          }
        }
      }
    }

  }

  _onMouseUp() {
    this._relax()
  }

  _onMouseMove(e) {
    if (this._mouseDown) {
      // TODO: update the scroll position
    } else {
      this._relax()
    }
  }

  _relax() {
    this._mouseDown = false
    if (platform.inBrowser) {
      let _window = DefaultDOMElement.wrap(window)
      _window.off(this)
    }
  }

  _onScroll() {
    this._updatePositions()
  }
}
