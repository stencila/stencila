const del = require('del')
const fs = require('fs')
const gulp = require('gulp')
const connect = require('gulp-connect')
const nunjucks = require('nunjucks')
const plumber = require('gulp-plumber')
const replaceExt = require('replace-ext')
const through = require('through2')
const yaml = require('js-yaml')

/**
 * Augment an OpenAPI 3.0 YAML specification with non-standard
 * vendor exntensions (e.g. `x-code-samples` enabled by redoc)
 */
function openapi2redoc () {
  // List of languages and thh templates to use to generate code samples 
  // for each of them
  const langs = ['Curl', 'Javascript', 'Python', 'R']
  const templates = {}
  for (let lang of langs) {
    templates[lang] = fs.readFileSync(`./site/request-code-samples/${lang.toLowerCase()}.txt`, 'utf8')
  }

  return through.obj(function(file, encoding, callback) {
    const content = file.contents.toString()
    const api = yaml.safeLoad(content, {json: true})
    const baseUrl = api.servers[0].url

    // Add code samples
    for (let [id, path] of Object.entries(api.paths)) {
      for (let [method, op] of Object.entries(path)) {
        const context = Object.assign({
          url: baseUrl + id,
          method
        }, op)
        let samples = []
        for (let lang of langs) {
          const source = nunjucks.renderString(templates[lang], context)
          samples.push({lang, source})
        }
        op['x-code-samples'] = samples
      }
    }

    const yml = yaml.dump(api)
    file.contents = Buffer.from(yml)
    file.path = replaceExt(file.path, '.redoc-yaml')
    callback(null, file)
  })
}

gulp.task('clean', function () {
  return del('./build')
})

gulp.task('site', function () {
  gulp.src([
    './site/*.{html,css,js}',
    './node_modules/redoc/bundles/redoc.standalone.js'
  ])
    .pipe(plumber())
    .pipe(gulp.dest('./build/'))
    .pipe(connect.reload())
})

gulp.task('src/openapi', function () {
  gulp.src(['./src/Host.yaml'])
    .pipe(plumber())
    .pipe(openapi2redoc())
    .pipe(gulp.dest('./build/'))
    .pipe(connect.reload())
})

gulp.task('src/yaml', function () {
  gulp.src(['./src/*.yaml'])
    .pipe(plumber())
    .pipe(gulp.dest('./build/'))
    .pipe(connect.reload())
})

gulp.task('build', ['clean'], function () {
  gulp.start([
    'site',
    'src/openapi',
    'src/yaml'
  ])
})

gulp.task('connect', function () {
  connect.server({
    root: 'build',
    livereload: true
  })
})

gulp.task('watch', function () {
  gulp.watch(['./site/**/*'], ['site'])
  gulp.watch(['./src/Host.yaml'], ['src/openapi'])
  gulp.watch(['./src/*.yaml'], ['src/yaml'])
})

gulp.task('default', ['build', 'connect', 'watch'])
