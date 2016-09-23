import Overlayer from './Overlayer'

export default {
  name: 'visualEditor',
  configure: function (config) {
    // Adds the 'overlay' component. This is necessary
    // config for how `ScrollPane` works but because of our inplementation
    // of an overlay class is actually unused
    config.addComponent('overlay', Overlayer)
  }
}
