export default `
<div id="title">Test with all nodes</div>
<p>A test Stencila document with at least one (well, that&apos;s the plan) of each node type (in alphabetical order). See additional node specific test documents in sibling folders e.g. <a href="/file://tests/document/nodes/image/index.html">image</a> </p>
<h1>Blockquote</h1>
<p>Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
<blockquote>Science is a way of thinking much more than it is a body of knowledge. <em>Carl Sagan</em></blockquote>
<p>Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
<h1>Codeblock</h1>
<p>Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
<pre><code class="r"># A R snippet (from the dplyr vignette)

by_tailnum &lt;- group_by(flights,=&quot;&quot; tailnum)=&quot;&quot; delay=&quot;&quot; &lt;-=&quot;&quot; summarise(by_tailnum,=&quot;&quot; count=&quot;n(),&quot; dist=&quot;mean(distance,&quot; na.rm=&quot;TRUE),&quot; filter(delay,=&quot;&quot;&gt; 20, dist &lt; 2000)

# Interestingly, the average delay is only slightly related to the
# average distance flown by a plane.
ggplot(delay, aes(dist, delay)) +
  geom_point(aes(size = count), alpha = 1/2) +
  geom_smooth() +
  scale_size_area()</code></pre>
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
<h1>Emphasis</h1>
<p>Some <em>emphasized text</em></p>
<h1>Heading</h1>
<h1>Link</h1>
<h1>Math</h1>
<h1>Execute directive</h1>
<h1>Strong</h1>
<p>Some <strong>strong text</strong></p>
<h1>Summary</h1>
<h1>Title</h1>
`
