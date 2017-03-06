export default `
<div id="title">Test with all nodes</div>
<p>A test Stencila document with at least one (well, that&apos;s the plan) of each node type (in alphabetical order). See additional node specific test documents in sibling folders e.g. <a href="/file://tests/document/nodes/image/index.html">image</a>.</p>
<h1>Math</h1>
<p>Here is some math: <span data-math="asciimath">sum_(i=1)^n i^3=((n(n+1))/2)^2</span>.</p>
<h1>Minilang Cell</h1>
<!-- Pure MiniLang cell -->
<div data-cell="barchart([4,8,15,16,23,42])">
  <pre data-output>
    <style>
    .chart div {
      font: 10px sans-serif;
      background-color: steelblue;
      text-align: right;
      padding: 3px;
      margin: 1px;
      color: white;
    }
    </style>
    <div class="chart">
      <div style="width: 40px;">4</div>
      <div style="width: 80px;">8</div>
      <div style="width: 150px;">15</div>
      <div style="width: 160px;">16</div>
      <div style="width: 230px;">23</div>
      <div style="width: 420px;">42</div>
    </div>
  </pre>
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
