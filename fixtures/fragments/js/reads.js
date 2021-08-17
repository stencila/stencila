import fs from 'fs'
import {readFileSync} from 'fs'

let data = fs.readFile('path1')
readFileSync('path2')

function readData() {
  fs.readFile("path3", {}, () => {
    readFileSync('path4')
  })
}
