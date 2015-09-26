// Gulp dependencies
var gulp = require('gulp');
var rename = require('gulp-rename');

// Build dependencies
var browserify = require('gulp-browserify');
var uglify = require('gulp-uglify');
var sass = require('gulp-sass');
var minifyCSS = require('gulp-minify-css');

// Test dependencies
var jasmine = require('gulp-jasmine');

// Types of components
var types = ['stencil'];

gulp.task('scripts', function() {
  types.forEach(function(type) {
    gulp.src('./stencila/'+type+'/'+type+'.js')
        .pipe(browserify())
        .pipe(uglify())
        .pipe(rename(type+'.min.js'))
        .pipe(gulp.dest('./build'));
  });
});

gulp.task('styles', function() {
  types.forEach(function(type) {
    gulp.src('./stencila/'+type+'/'+type+'.scss')
        .pipe(sass().on('error', sass.logError))
        .pipe(minifyCSS())
        .pipe(rename(type+'.min.css'))
        .pipe(gulp.dest('./build'));
  });
});

gulp.task('build', ['scripts','styles']);

gulp.task('test', ['build'], function () {
  return gulp.src('tests/jasmine/**/*.js')
    .pipe(jasmine());
});


gulp.task('default', ['build','test']);
