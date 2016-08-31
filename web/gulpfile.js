// Gulp dependencies
var gulp = require('gulp');
var gutil = require('gulp-util');
var notify = require('gulp-notify');
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


// Test dependencies
var jasmine = require('gulp-jasmine');

// Types of components
var types = [
  'document/document',
  'stencil/stencil','stencil/stencil-strict','stencil/stencil-free',
  'sheet/sheet'
];

// Generic error handler creates a notifcation window
function errorHandler() {
  var args = Array.prototype.slice.call(arguments);
  notify.onError({
    title: 'Compile Error',
    message: '<%= error.message %>'
  }).apply(this, args);
  this.emit('end'); // Keep gulp from hanging on this task
}


function style(type,watch) {
  var src = './'+type+'.scss';
  var dest = type.split('/')[1]+'.min.css';

  gulp.src(src)
    .pipe(sass({
        outputStyle: watch?'expanded':'compressed',
        includePaths: require('node-normalize-scss').includePaths
     })
    .on('error', errorHandler))
    .pipe(rename(dest))
    .pipe(gulp.dest('./build'));
}

function styles(watch) {
  gutil.log('Compiling styles');
  types.forEach(function(type) {
    style(type,watch);
  });
}

// Scripts watchify-browserify-babelify-uglify-sourcemapify
// Thanks to 
//  https://gist.github.com/wesbos/52b8fe7e972356e85b43
//  https://gist.github.com/danharper/3ca2273125f500429945
// and others  
function script(type,watch) {
  var src = './'+type+'.js';
  var dest = type.split('/')[1]+'.min.js';

  var props = {
    entries: [src],
    debug : true,
    transform:  [babelify]
  };

  var bundler = watch ? watchify(browserify(props)) : browserify(props);

  function bundle() {
    return bundler
      .bundle()
      .on('error',errorHandler)
      .pipe(source(src))
      .pipe(buffer())
      .pipe(rename(dest))
      .pipe(sourcemaps.init({
        loadMaps: true
      }))
      .pipe(gif(!watch, uglify()))
      .pipe(sourcemaps.write('.'))
      .pipe(gulp.dest('./build'));
  }

  bundler.on('update', function() {
    bundle();
    gutil.log('Bundling scripts');
  });

  return bundle();
}

function scripts(watch) {
  gutil.log('Bundling scripts');
  types.forEach(function(type) {
    script(type,watch);
  });
}

function images(watch) {
  gutil.log('Copying images');
  gulp.src('./images/**/*.{png,svg}')
      .pipe(gulp.dest('./build/images'));
}

function fonts(watch) {
  gutil.log('Copying fonts');
  gulp.src('./fonts/**/*')
      .pipe(gulp.dest('./build/fonts'));
}

// Gulp tasks for the above

gulp.task('styles', function() {
  return styles();
});

gulp.task('scripts', function() {
  return scripts();
});

gulp.task('images', function() {
  return images();
});

gulp.task('fonts', function() {
  return fonts();
});

gulp.task('build', function() {
  styles();
  scripts();
  images();
  fonts();
});

gulp.task('watch', function() {
  gulp.watch('**/*.scss', function(){
    styles(true);
  });
  scripts(true);
});

gulp.task('lint:sass', function () {
  return gulp.src([
    './document/*.scss'
  ])
  .pipe(sassLint({
    //configFile: '.sass-lint.yml'
  }))
  .pipe(sassLint.format())
  .pipe(sassLint.failOnError());
});

gulp.task('lint', [
  'lint:sass'
]);

gulp.task('test', ['build'], function () {
  return gulp.src('tests/jasmine/**/*.js')
    .pipe(jasmine());
});

gulp.task('default', ['watch']);
