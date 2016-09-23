var mockery = require('mockery')

mockery.registerMock('redis', require('fakeredis'))
mockery.enable({
  warnOnReplace: false,
  warnOnUnregistered: false
})
