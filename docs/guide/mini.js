// A temporary JS file with embedded HTML (!) to be replaced by Markdown
export default `

<div data-title>Mini</div>

<p>Stencila has it's own language for reproducible, data-drive, research called <em>Mini</em>. An important aspect of reproducibility is readability. Mini aims to be as easy to read and understand as it is to write. Mini is meant to be only slightly more advanced that the expressions that your write in your calculator or into the cell of a spreadsheet</p>

<p>Mini is intended as a glue language</p>

<div data-cell="run()" data-language="js">
  <pre data-source>context_variable = 42</pre>
</div>

<div data-cell="run()" data-language="js">
  <pre data-source>context_variable * 2</pre>
</div>

<p>Mini is intended as a glue language</p>

<div data-cell="call()" data-language="js">
  <pre data-source>return Math.random()</pre>
</div>

<p>If the radius of a circle is: <input name="radius" type="range" min="0" max="100" step="1" value="50"></input>m.</p>

<div data-cell="area = call(radius)" data-language="js">
  <pre data-source>return 2*Math.PI*Math.pow(radius, 2)</pre>
</div>

<p>Then it's area is: <span data-cell="area"></span>m<sup>2</sup>.</p>


<h1>Types</h1>

<p>Mini has a simple type system that is similar to most high level languages. Each type can be constructed using literals...</p>

<table>
  <tr><td>Type</td><td>Example literal</td></tr>
  <tr><td>null</td><td><code>null</code></td></tr>
  <tr><td>boolean</td><td><code>true, false</code></td></tr>
  <tr><td>integer</td><td><code>1, 2, 42</code></td></tr>
  <tr><td>float</td><td><code>3.13</code></td></tr>
  <tr><td>array</td><td><code>[1, 2, 3]</code></td></tr>
  <tr><td>object</td><td><code>{ a: 1, b: '2'}</code></td></tr>
  <tr><td>my_custom_type</td><td><code>{ type: 'my_custom_type', ...}</code></td>></tr>
</table>

<p>You can get a </p>

<div data-celloooooooooooo="type({})"></div>


<div data-cell="data = table({
species: [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3],
sepal_length: [5.1,4.9,4.7,4.6,5,5.4,4.6,5,4.4,4.9,5.4,4.8,4.8,4.3,5.8,5.7,5.4,5.1,5.7,5.1,5.4,5.1,4.6,5.1,4.8,5,5,5.2,5.2,4.7,4.8,5.4,5.2,5.5,4.9,5,5.5,4.9,4.4,5.1,5,4.5,4.4,5,5.1,4.8,5.1,4.6,5.3,5,7,6.4,6.9,5.5,6.5,5.7,6.3,4.9,6.6,5.2,5,5.9,6,6.1,5.6,6.7,5.6,5.8,6.2,5.6,5.9,6.1,6.3,6.1,6.4,6.6,6.8,6.7,6,5.7,5.5,5.5,5.8,6,5.4,6,6.7,6.3,5.6,5.5,5.5,6.1,5.8,5,5.6,5.7,5.7,6.2,5.1,5.7,6.3,5.8,7.1,6.3,6.5,7.6,4.9,7.3,6.7,7.2,6.5,6.4,6.8,5.7,5.8,6.4,6.5,7.7,7.7,6,6.9,5.6,7.7,6.3,6.7,7.2,6.2,6.1,6.4,7.2,7.4,7.9,6.4,6.3,6.1,7.7,6.3,6.4,6,6.9,6.7,6.9,5.8,6.8,6.7,6.7,6.3,6.5,6.2,5.9],
sepal_width: [3.5,3,3.2,3.1,3.6,3.9,3.4,3.4,2.9,3.1,3.7,3.4,3,3,4,4.4,3.9,3.5,3.8,3.8,3.4,3.7,3.6,3.3,3.4,3,3.4,3.5,3.4,3.2,3.1,3.4,4.1,4.2,3.1,3.2,3.5,3.6,3,3.4,3.5,2.3,3.2,3.5,3.8,3,3.8,3.2,3.7,3.3,3.2,3.2,3.1,2.3,2.8,2.8,3.3,2.4,2.9,2.7,2,3,2.2,2.9,2.9,3.1,3,2.7,2.2,2.5,3.2,2.8,2.5,2.8,2.9,3,2.8,3,2.9,2.6,2.4,2.4,2.7,2.7,3,3.4,3.1,2.3,3,2.5,2.6,3,2.6,2.3,2.7,3,2.9,2.9,2.5,2.8,3.3,2.7,3,2.9,3,3,2.5,2.9,2.5,3.6,3.2,2.7,3,2.5,2.8,3.2,3,3.8,2.6,2.2,3.2,2.8,2.8,2.7,3.3,3.2,2.8,3,2.8,3,2.8,3.8,2.8,2.8,2.6,3,3.4,3.1,3,3.1,3.1,3.1,2.7,3.2,3.3,3,2.5,3,3.4,3]
})"></div>

<h1>Plotting</h1>

<p>Mini has several plotting functions</p>


<div data-cell="points(data, 'sepal_length', 'sepal_width')"></div>


<div data-cell="points(data, 'sepal_length', 'sepal_width', color='species', options={
  encoding:{
    color:{
      legend:{title:'Species'}
    }
  }
})"></div>

<div data-cell="titles(points(data, 'sepal_length', 'sepal_width', color='species'), color='Species')"></div>

<table>
  <tr><td>Type</td><td>Example literal</td></tr>
  <tr><td>point</td><td><code>null</code></td></tr>
</table>

`