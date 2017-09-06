import {
  DefaultDOMElement, Component,
  platform
} from 'substance'

export default class SheetScrollbar extends Component {

  didMount() {
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
  }

  dispose() {
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
  }

  didUpdate() {
    this._updatePositions()
  }

  render($$) {
    const horizontal = this._isHorizontal()
    let el = $$('div')
      .addClass('sc-sheet-scrollbar')
      .addClass(horizontal ? 'sm-horizontal' : 'sm-vertical')
      .on('mousedown', this._onMouseDown)
    let thumb = $$('div').ref('thumb').addClass('se-thumb')
    el.append(thumb)
    return el
  }

  _isHorizontal() {
    return this.props.axis === 'x'
  }

  _updatePositions() {
    const sheet = this.props.sheet
    const viewport = this.props.viewport
    const axis = this.props.axis
    let contentSize, widgetSize, scrollPos
    let horizontal = this._isHorizontal()
    if (horizontal) {
      contentSize = sheet.getColumnCount()*viewport.D
      widgetSize = viewport.W
      scrollPos = viewport.x
    } else {
      contentSize = sheet.getRowCount()*viewport.D
      widgetSize = viewport.H
      scrollPos = viewport.y
    }
    if (contentSize < widgetSize) {
      this.el.addClass('sm-hide-thumb')
    } else {
      this.el.removeClass('sm-hide-thumb')
      let pos = (scrollPos / contentSize) * widgetSize
      let size = (viewport.D / contentSize) * widgetSize
      if (horizontal) {
        this.refs.thumb.css({
          left: pos,
          width: size
        })
      } else {
        this.refs.thumb.css({
          top: pos,
          height: size
        })
      }
    }
  }

  _onResize() {
    // do a full rerender when window gets resized
    this.rerender()
  }

  _onMouseDown(e) {
    e.stopPropagation()
    e.preventDefault()
    this._mouseDown = true
    if (platform.inBrowser) {
      // temporarily, we bind to events on window level
      // because could leave the this element's area while dragging
      let _window = DefaultDOMElement.wrap(window)
      _window.on('mousemove', this._onMouseMove, this)
      _window.on('mouseup', this._onMouseUp, this)
    }

    // TODO: if clicked outside of the thumb
    // scroll to the position directly
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
}
