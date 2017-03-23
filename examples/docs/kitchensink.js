export default `
<div data-title>Test with all nodes 😃 🇳🇿</div>
<p>A test Stencila document with at least one (well, that&apos;s the plan) of each node type (in alphabetical order). See additional node specific test documents in sibling folders e.g. <a href="/file://tests/document/nodes/image/index.html">image</a>.</p>
<h1>Input</h1>
<p>If I eat <input type="range" min="1" max="20" step="1" name="num_cookies" value="3"/> extra <select name="calories"><option value="30.2">Fortune</option><option value="71.6" selected="true">Peanut butter</option><option value="65.3">Oatmeal</option></select> (<span data-cell="calories">71.6</span> calories each) every day for the next <input type="range" min="1" max="100" step="1" name="num_weeks" value="52"/> weeks that will be <span data-cell="num_cookies*calories*num_weeks">11949.60</span> extra calories total.</p>
<h1>Math</h1>
<p>Here is some math: <span data-math="asciimath">sum_(i=1)^n i^3=((n(n+1))/2)^2</span>.</p>
<!-- Vegalite example -->
<div data-cell="a_height = 15"></div>
<div data-cell="call(a_height)" data-language="js">
<pre data-source>
return {"type":"vegalite","data":{"values":[{"type":"A","height":a_height},{"type":"B","height":55},{"type":"C","height":43}]},"mark":"bar","encoding":{"x":{"field":"type","type":"ordinal"},"y":{"field":"height","type":"quantitative"}}}
</pre>
</div>

<h1>Minilang Cell</h1>

<!-- Pure MiniLang cell -->
<div data-cell='mytable = { type: "table", data: { x: [10,11,12], y: [20,21,22] } }'></div>
<div data-cell="x = type(1)"></div>
<p>Here is an external cell, implemented in Javascript:</p>
<div data-cell="length = 5"></div>
<div data-cell="random_numbers = call(length)" data-language="js">
<pre data-source>var randomNumbers = []
for (var i = 0; i < length; i++) {
  randomNumbers.push(Math.floor(Math.random()*100))
}
return randomNumbers</pre>
</div>
<h1>Blockquote</h1>
<p>Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
<blockquote>Science is a way of thinking much more than it is a body of knowledge. <em>Carl Sagan</em></blockquote>
<p>Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
<h1>Lists</h1>
<ol>
  <li>First</li>
  <li>Second</li>
  <li>Third</li>
</ol>
<ul>
  <li>First</li>
  <li>Second</li>
  <li>Third</li>
</ul>
<h1>Table</h1>
<table>
  <tbody>
    <tr><td>A1</td><td>B1</td><td>C1</td></tr>
    <tr><td>A2</td><td>B2</td><td>C2</td></tr>
  </tbody>
</table>
<h1>Emphasis</h1>
<p>Some <em>emphasized text</em></p>
<h1>Heading</h1>
<h1>Link</h1>
<h1>Execute directive</h1>
<h1>Strong</h1>
<p>Some <strong>strong text</strong></p>
<h1>Summary</h1>
<h1>Title</h1>
`
