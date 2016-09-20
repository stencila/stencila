// Gulp dependencies
var gulp = require('gulp');
var gutil = require('gulp-util');
var rename = require('gulp-rename');
var gif = require('gulp-if');
var sourcemaps = require('gulp-sourcemaps');
var source = require('vinyl-source-stream');
var buffer = require('vinyl-buffer');

// Build dependencies
var browserify = require('browserify');
var watchify = require('watchify');
var babelify = require('babelify');
var uglify = require('gulp-uglify');
var sass = require('gulp-sass');
var sassLint = require('gulp-sass-lint');
var eslint = require('gulp-eslint');

// Types of components
var types = [
  'document'
];

// Generic error handler creates a notifcation window
function errorHandler (err) {
  gutil.log(err);
  this.emit('end'); // Keep gulp from hanging on this task
}

function style (type, watch) {
  gulp.src('./' + type + '/' + type + '.scss')
    .pipe(sass({
      outputStyle: watch ? 'expanded' : 'compressed'
    })
    .on('error', errorHandler))
    .pipe(rename(type + '.min.css'))
    .pipe(gulp.dest('./build'));
}

function styles (watch) {
  gutil.log('Compiling styles');
  types.forEach(function (type) {
    style(type, watch);
  });
}

// Scripts watchify-browserify-babelify-uglify-sourcemapify
// Thanks to
//  https://gist.github.com/wesbos/52b8fe7e972356e85b43
//  https://gist.github.com/danharper/3ca2273125f500429945
// and others
function script (type, watch) {
  var bundler = browserify('./' + type + '/' + type + '.js', {
    debug: true
  });

  function bundle () {
    return bundler
      .transform(babelify, {
        presets: ['es2015']
      })
      .bundle()
      .on('error', errorHandler)
      .pipe(source('./' + type + '.min.js'))
      .pipe(buffer())
      .pipe(sourcemaps.init({
        loadMaps: true
      }))
      .pipe(gif(!watch, uglify()))
      .pipe(sourcemaps.write('.'))
      .pipe(gulp.dest('./build'));
  }

  if (watch) {
    bundler = watchify(bundler);
    bundler.on('update', function () {
      gutil.log('Bundling ' + type);
      bundle();
    });
    bundle();
  } else {
    return bundle();
  }
}

function scripts (watch) {
  gutil.log('Bundling scripts');
  types.forEach(function (type) {
    script(type, watch);
  });
}

function images (watch) {
  gutil.log('Copying images');
  gulp.src('./images/**/*.{png,svg}')
      .pipe(gulp.dest('./build/images'));
}

function fonts (watch) {
  gutil.log('Copying fonts');
  gulp.src('./fonts/**/*')
      .pipe(gulp.dest('./build/fonts'));
}

// Gulp tasks for the above

gulp.task('styles', function () {
  return styles();
});

gulp.task('scripts', function () {
  return scripts();
});

gulp.task('images', function () {
  return images();
});

gulp.task('fonts', function () {
  return fonts();
});

gulp.task('build', function () {
  styles();
  scripts();
  images();
  fonts();
});

gulp.task('watch', function () {
  gulp.watch('**/*.scss', function () {
    styles(true);
  });
  scripts(true);
});

gulp.task('lint:js', function () {
  return gulp.src([
    './*.js',
    './collab/**/*.js',
    './document/**/*.js',
    './shared/**/*.js',
    './tests/**/*.js'
  ]).pipe(eslint())
    .pipe(eslint.format())
    .pipe(eslint.failAfterError());
});

gulp.task('lint:sass', function () {
  return gulp.src([
    './document/**/*.scss'
  ])
  .pipe(sassLint({
    files: {
      ignore: 'document/_resets.scss'
    }
  }))
  .pipe(sassLint.format())
  .pipe(sassLint.failOnError());
});

gulp.task('lint', [
  'lint:js',
  'lint:sass'
]);

gulp.task('default', ['watch']);
