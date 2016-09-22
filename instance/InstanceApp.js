import Component from 'substance/ui/Component'

class SessionApp extends Component {

  getInitialState () {
    return {
    }
  }

  render ($$) {
    var el = $$('div').addClass('sc-document-app')

    if (!this.props.data.components) {
      el.append(this.renderLogin($$));
    } else {
      el.html('Home')
    }

    return el
  }

  renderLogin ($$) {
    return $$('div')
      .html(
        `<div class="ui middle aligned center aligned grid" style="height:80%">
          <div class="ui column" style="max-width: 30em">

            <div class="ui top attached message">
              <div clas="ui middle aligned small image">
                <img src="/web/images/logo.svg">
              </div>
            </div>

            <form id="login-form" class="ui attached fluid form segment">
                <div class="ui field">
                  <div class="ui left icon input">
                    <i class="ui lock icon"></i>
                    <input type="token" name="token" placeholder="Token">
                  </div>
                </div>
                <button type="submit" class="ui fluid green submit button">Login</button>
            </form>

            <div class="ui small bottom attached message">
              <i class="ui help icon"></i>
              You can get the token by entering <code>stencila.session.token</code> in your Python session.
            </div>

          </div>
        </div>`
      )
  }

}

export default SessionApp
