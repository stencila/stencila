export default class Storer {

  readFile (path) { // eslint-disable-line
    throw new Error('Storer.readFile() must be implemented in derived class')
  }
  
  writeFile (path, data) { // eslint-disable-line
    throw new Error('Storer.writeFile() must be implemented in derived class')
  }

  readDir (path) { // eslint-disable-line
    throw new Error('Storer.readDir() must be implemented in derived class')
  }

}
