import redis from 'redis'

class Store {

  constructor () {
    // TODO Share client across stores
    // TODO Read in a Redis config
    var config = {
      redis: {
        // host
        // port
      }
    }
    this.client = redis.createClient(config.redis)
  }

}

export default Store
