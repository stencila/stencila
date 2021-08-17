import fs from 'fs'
import {writeFileSync} from 'fs'

let data = fs.writeFile('path1', data)
writeFileSync('path2', data, 'utf-8')

function writeData() {
  fs.writeFile("path3", data)
}
