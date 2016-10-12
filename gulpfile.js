// Gulp dependencies
var gulp = require('gulp')
var gutil = require('gulp-util')
var rename = require('gulp-rename')
var gif = require('gulp-if')
var sourcemaps = require('gulp-sourcemaps')
var source = require('vinyl-source-stream')
var buffer = require('vinyl-buffer')

// Build dependencies
var browserify = require('browserify')
var watchify = require('watchify')
var babelify = require('babelify')
var uglify = require('gulp-uglify')
var sass = require('gulp-sass')
var sassLint = require('gulp-sass-lint')
var eslint = require('gulp-eslint')
var rm = require('gulp-rm')

// Types of components
var types = [
  'document'
]

function style (type, dev) {
  gulp.src('./' + type + '/' + type + '.scss')
    .pipe(sass({
      outputStyle: dev ? 'expanded' : 'compressed'
    })
    .on('error', function (err) {
      gutil.log(err)
      this.emit('end') // Keep gulp from hanging on this task
    }))
    .pipe(rename(type + '.min.css'))
    .pipe(gulp.dest('./build'))
}

function styles (dev) {
  gutil.log('Compiling styles')
  types.forEach(function (type) {
    style(type, dev)
  })
}

/**
 * Get a script bundler for a source file
 *
 * @param  {string} source The source file to bundle
 * @return {Object}        A browserify bundler
 */
function bundler (source, dev) {
  return browserify(source, {
    debug: dev,
    cache: {},
    packageCache: {}
  }).transform(babelify, {
    presets: ['es2015'],
    // Substance needs to be transformed
    global: true,
    ignore: /\/node_modules\/(?!substance\/)/
  })
}

let bundlers = {
  'document': bundler('document/document.js')
}

function script (type, dev) {
  let bundler = bundlers[type]
  function bundle () {
    gutil.log('Bundling ' + type + '.js')
    return bundler
      .bundle()
      .on('error', function (err) {
        gutil.log(err)
        this.emit('end') // Keep gulp from hanging on this task
      })
      .pipe(source('./' + type + '.min.js'))
      .pipe(buffer())
      //.pipe(sourcemaps.init({
      //  loadMaps: true
      //}))
      //.pipe(gif(!dev, uglify()))
      //.pipe(sourcemaps.write('.'))
      .pipe(gulp.dest('./build'))
  }

  if (dev) {
    bundler = watchify(bundler)
    bundler.on('update', function () {
      bundle()
    })
    bundle()
  } else {
    return bundle()
  }
}

function scripts (dev) {
  gutil.log('Bundling scripts')
  types.forEach(function (type) {
    script(type, dev)
  })
}

function images () {
  gutil.log('Copying images')
  gulp.src('./images/**/*.{png,svg}')
      .pipe(gulp.dest('./build/images'))
  gulp.src('./node_modules/emojione/assets/png/*')
      .pipe(gulp.dest('./build/emojione/png'))
}

function fonts () {
  gutil.log('Copying fonts')
  gulp.src('./fonts/**/*')
      .pipe(gulp.dest('./build/fonts'))
  gulp.src('./node_modules/font-awesome/fonts/*')
      .pipe(gulp.dest('./build/fonts'))
}

function lintJs () {
  gutil.log('Linting JS')
  gulp.src([
    './*.js',
    './collab/**/*.js',
    './document/**/*.js',
    './tests/**/*.js',
    './utilities/**/*.js'
  ]).pipe(eslint())
    .pipe(eslint.format())
    .pipe(eslint.failAfterError())
}

function lintSass () {
  gutil.log('Linting SASS')
  gulp.src([
    './document/**/*.scss'
  ])
  .pipe(sassLint({
    files: {
      ignore: 'document/_resets.scss'
    }
  }))
  .pipe(sassLint.format())
  .pipe(sassLint.failOnError())
}

function clean () {
  return gulp.src('build/**/*', {read: false})
    .pipe(rm())
}

// Gulp tasks

gulp.task('styles', function () {
  styles()
})

gulp.task('scripts', function () {
  scripts()
})

gulp.task('images', function () {
  images()
})

gulp.task('fonts', function () {
  fonts()
})

gulp.task('lint:js', function () {
  lintJs()
})

gulp.task('lint:sass', function () {
  lintSass()
})

gulp.task('lint', [
  'lint:js', 'lint:sass'
])

gulp.task('build', [
  'styles', 'scripts', 'fonts', 'images'
])

gulp.task('clean', function () {
  clean()
})

gulp.task('watch', function () {
  gulp.watch('**/*.scss', function () {
    styles(true)
  })
  scripts(true)
})

gulp.task('default', ['build', 'watch'])
