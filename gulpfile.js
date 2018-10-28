const connect = require('gulp-connect')
const del = require('del')
const fs = require('fs')
const gulp = require('gulp')
const nunjucks = require('nunjucks')
const plumber = require('gulp-plumber')
const replaceExt = require('replace-ext')
const through = require('through2')
const typedoc = require('typedoc')
const yaml = require('js-yaml')

/**
 * Convert TypeScript to TypeDoc HTML and JSON
 * output
 */
function ts2typedoc () {
  // Run TypeDoc...
  const app = new typedoc.Application({
      // TypeScript options (should be same as in `tsconfig.json`)
      module: 'commonjs',
      target: 'es2017',
      experimentalDecorators: true,
      // TypeDoc options (see https://typedoc.org/api/)
      name: 'Stencila Schema',
      mode: 'file',
      readme: 'README.md'
  })
  const project = app.convert(app.expandInputFiles(['./src']))
  // Write out HTML
  app.generateDocs(project, './build')
  // Return project object for further processing
  return project
}

/**
 * Convert TypeDoc JSON to JSON-LD context
 * 
 * Examples of other context definitions:
 * 
 *   http https://schema.org/docs/jsonldcontext.json | less
 *   http https://schema.org/Thing.jsonld
 *   http https://raw.githubusercontent.com/codemeta/codemeta/2.0/codemeta.jsonld
 *   http https://science.ai Accept:application/ld+json
 */
function typedoc2context (docs) {
  const context = {
    // Contexts referrred to
    schema: 'https://schema.org/',
    rdfs: 'http://www.w3.org/2000/01/rdf-schema#',
    codemeta: 'https://doi.org/10.5063/schema/codemeta-2.0',
    stencila: 'https://stencila.github.io/schema/context.jsonld',

    // Alias type and id
    // See "Addressing the “@” issue" at https://datalanguage.com/news/publishing-json-ld-for-developers
    // for why
    // This is done in the https://schema.org/ context, so if we are extending schema.org
    // is this necessary?
    type: '@type',
    id: '@id',
  }

  // Iterate over classes
  for (let clas of docs.children.filter(clas => clas.kindString === 'Class')) {
    // Only include registered types (i.e. having the `@type` decorator)
    let type = clas.decorators && clas.decorators.filter(dec => dec.name === 'type')[0]
    if (type) {
      let id = type.arguments.id.replace(/\'|\"/g, '')
      let [cxt, name] = id.split(':')
      if (cxt !== 'stencila') {
        // Type defined in another context
        context[clas.name] = {
          '@id': id
        }
      } else {
        // No `@id` tag so define a new class
        const type = {
          '@id': clas.name, 
          '@type': 'rdfs:Class', 
          'rdfs:label': clas.name
        }
        let comment = clas.comment && clas.comment.shortText
        if (comment) {
          type['rdfs:comment'] = comment
        }
        context[clas.name] = type
      }
    }
  }
  
  // Write to file
  const json = JSON.stringify({'@context': context}, null, '  ')
  fs.writeFileSync('./build/context.jsonld', json)  
}

/**
 * Augment an OpenAPI 3.0 YAML specification with non-standard
 * vendor extensions (e.g. `x-code-samples` enabled by redoc)
 */
function openapi2redoc () {
  // List of languages and the templates to use to generate code samples 
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
    file.path = replaceExt(file.path, '.redoc.yaml')
    callback(null, file)
  })
}

/**
 * Convert a YAML file to a JSON file e.g. so that is can be referenced
 * from other JSON Schema documents
 */
function yaml2json () {
  return through.obj(function(file, encoding, callback) {
    const yml = file.contents.toString()
    const doc = yaml.safeLoad(yml, {json: true})
    const json = JSON.stringify(doc, null, '  ')
    file.contents = Buffer.from(json)
    file.path = replaceExt(file.path, '.json')
    callback(null, file)
  })
}

gulp.task('site', function () {
  gulp.src([
    './site/*.{html,css,js}',
    './node_modules/redoc/bundles/redoc.standalone.js'
  ])
    .pipe(plumber())
    .pipe(gulp.dest('./build/'))
    .pipe(connect.reload())
})

gulp.task('src/ts', function () {
  const docs = ts2typedoc()
  typedoc2context(docs)
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
    .pipe(yaml2json())
    .pipe(gulp.dest('./build/'))
    .pipe(connect.reload())
})

gulp.task('build', ['clean'], function () {
  gulp.start([
    'site',
    'src/ts',
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

gulp.task('clean', function () {
  return del('./build')
})

gulp.task('default', ['build', 'connect', 'watch'])
