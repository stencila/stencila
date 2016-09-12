'use strict';

var test = require('tape');

var math = require('../../../utilities/math');

test('math.translate Tex to TeX', function (t) {
  t.equal(math.translate('\\pi', 'tex', 'tex'), '\\pi');
  t.end();
});

test('math.translate ASCIIMath to TeX', function (t) {
  [
    ['', ''],
    ['   ', ''],
    ['\t', ''],

    ['1', '{1}'],

    ['pi', '\\pi'],
    ['omega', '\\omega'],
    ['oo', '\\infty'],

    ['a+b', '{a}+{b}'],
    ['a-b', '{a}-{b}'],
    ['a*b', '{a}\\cdot{b}'],
    ['a**b', '{a}\\ast{b}'],
    ['a***b', '{a}\\star{b}'],
    ['axxb', '{a}\\times{b}'],
    ['a/b', '\\frac{{a}}{{b}}'],
    ['a//b', '{a}/{b}'],
    ['a-:b', '{a}\\div{b}'],

    ['a_1', '{a}_{{1}}'],
    ['a^1', '{a}^{{1}}'],
    ['a_1^2', '{{a}_{{1}}^{{2}}}'],
    ['a_{1,2}', '{a}_{{{1},{2}}}'],
    ['a_(1,2)', '{a}_{{{1},{2}}}'],

    ['sum x', '\\sum{x}'],
    ['prod x', '\\prod{x}'],
    ['sqrt x', '\\sqrt{{x}}'],
    ['sqrt{x+y}', '\\sqrt{{{x}+{y}}}'],

    ['hat x bar x vec x', '\\hat{{x}}\\overline{{x}}\\vec{{x}}'],

    ['root a b', '{\\sqrt[{a}]{{b}}}'],
    ['abs -1', '{\\left|-\\right|}{1}'],
    ['cancel', '\\cancel{}'],

    ['|_', '{\\mid}_{}'],
    ['_|', '_{\\mid}'],

    ['and', '{\\quad\\text{and}\\quad}'],
    ['if', '{\\quad\\text{if}\\quad}'],
    ['iff', '\\Leftrightarrow'],

    ['[[a,b],[c,d]]', '{\\left[\\matrix{{a}&{b}\\\\{c}&{d}}\\right]}'],
    ['((a,b),(c,d))', '{\\left(\\matrix{{a}&{b}\\\\{c}&{d}}\\right)}'],

    ['"Aa"', '\\text{Aa}'],
    ['bb "Aa"', '{\\mathbf{\\text{Aa}}}']

  ].forEach(function (pair) {
    t.equal(math.translate(pair[0], 'am', 'tex'), pair[1]);
  });
  t.end();
});

test('math.render', function (t) {
  var html;
  html = math.render('x');
  var start = '<span class="katex"><span class="katex-mathml"><math>';
  t.equal(html.substr(0, start.length), start);
  t.end();
});
