(function() {var type_impls = {
"geo_types":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#80-101\">source</a><a href=\"#impl-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.x_y\" class=\"method\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#98-100\">source</a><h4 class=\"code-header\">pub fn <a href=\"geo_types/geometry/struct.Coord.html#tymethod.x_y\" class=\"fn\">x_y</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T, T)</a></h4></section></summary><div class=\"docblock\"><p>Returns a tuple that contains the x/horizontal &amp; y/vertical component of the coordinate.</p>\n<h5 id=\"examples\"><a href=\"#examples\">Examples</a></h5>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>c = <span class=\"macro\">coord! </span>{\n    x: <span class=\"number\">40.02f64</span>,\n    y: <span class=\"number\">116.34</span>,\n};\n<span class=\"kw\">let </span>(x, y) = c.x_y();\n\n<span class=\"macro\">assert_eq!</span>(y, <span class=\"number\">116.34</span>);\n<span class=\"macro\">assert_eq!</span>(x, <span class=\"number\">40.02f64</span>);</code></pre></div>\n</div></details></div></details>",0,"geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#249-257\">source</a><a href=\"#impl-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"docblock\"><p>Create a coordinate at the origin.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::Coord;\n<span class=\"kw\">use </span>num_traits::Zero;\n\n<span class=\"kw\">let </span>p: Coord = Zero::zero();\n\n<span class=\"macro\">assert_eq!</span>(p.x, <span class=\"number\">0.</span>);\n<span class=\"macro\">assert_eq!</span>(p.y, <span class=\"number\">0.</span>);</code></pre></div>\n</div><div class=\"impl-items\"><section id=\"method.zero\" class=\"method\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#251-256\">source</a><h4 class=\"code-header\">pub fn <a href=\"geo_types/geometry/struct.Coord.html#tymethod.zero\" class=\"fn\">zero</a>() -&gt; Self</h4></section></div></details>",0,"geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Add-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#147-157\">source</a><a href=\"#impl-Add-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"docblock\"><p>Add two coordinates.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>p = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.25</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>q = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.5</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>sum = p + q;\n\n<span class=\"macro\">assert_eq!</span>(sum.x, <span class=\"number\">2.75</span>);\n<span class=\"macro\">assert_eq!</span>(sum.y, <span class=\"number\">5.0</span>);</code></pre></div>\n</div><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>The resulting type after applying the <code>+</code> operator.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.add\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#151-156\">source</a><a href=\"#method.add\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html#tymethod.add\" class=\"fn\">add</a>(self, rhs: Self) -&gt; Self</h4></section></summary><div class='docblock'>Performs the <code>+</code> operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html#tymethod.add\">Read more</a></div></details></div></details>","Add","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Div%3CT%3E-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#223-233\">source</a><a href=\"#impl-Div%3CT%3E-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;T&gt; for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"docblock\"><p>Divide coordinate wise by a scalar.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>p = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">5.</span>, y: <span class=\"number\">10. </span>};\n<span class=\"kw\">let </span>q = p / <span class=\"number\">4.</span>;\n\n<span class=\"macro\">assert_eq!</span>(q.x, <span class=\"number\">1.25</span>);\n<span class=\"macro\">assert_eq!</span>(q.y, <span class=\"number\">2.5</span>);</code></pre></div>\n</div><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>The resulting type after applying the <code>/</code> operator.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.div\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#227-232\">source</a><a href=\"#method.div\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html#tymethod.div\" class=\"fn\">div</a>(self, rhs: T) -&gt; Self</h4></section></summary><div class='docblock'>Performs the <code>/</code> operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html#tymethod.div\">Read more</a></div></details></div></details>","Div<T>","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3C%5BT;+2%5D%3E-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#46-54\">source</a><a href=\"#impl-From%3C%5BT;+2%5D%3E-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.array.html\">[T; 2]</a>&gt; for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#48-53\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(coords: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.array.html\">[T; 2]</a>) -&gt; Self</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<[T; 2]>","geo_types::geometry::coord::Coordinate"],["<section id=\"impl-Copy-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Copy-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section>","Copy","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Default-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","geo_types::geometry::coord::Coordinate"],["<section id=\"impl-StructuralPartialEq-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-StructuralPartialEq-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section>","StructuralPartialEq","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3C(T,+T)%3E-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#36-44\">source</a><a href=\"#impl-From%3C(T,+T)%3E-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T, T)</a>&gt; for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#38-43\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(coords: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T, T)</a>) -&gt; Self</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<(T, T)>","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Debug-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Clone-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Neg-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#118-131\">source</a><a href=\"#impl-Neg-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Neg.html\" title=\"trait core::ops::arith::Neg\">Neg</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Neg.html\" title=\"trait core::ops::arith::Neg\">Neg</a>&lt;Output = T&gt;,</span></h3></section></summary><div class=\"docblock\"><p>Negate a coordinate.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>p = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.25</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>q = -p;\n\n<span class=\"macro\">assert_eq!</span>(q.x, -p.x);\n<span class=\"macro\">assert_eq!</span>(q.y, -p.y);</code></pre></div>\n</div><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Neg.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>The resulting type after applying the <code>-</code> operator.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.neg\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#125-130\">source</a><a href=\"#method.neg\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Neg.html#tymethod.neg\" class=\"fn\">neg</a>(self) -&gt; Self</h4></section></summary><div class='docblock'>Performs the unary <code>-</code> operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Neg.html#tymethod.neg\">Read more</a></div></details></div></details>","Neg","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3CPoint%3CT%3E%3E-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#56-64\">source</a><a href=\"#impl-From%3CPoint%3CT%3E%3E-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"geo_types/geometry/struct.Point.html\" title=\"struct geo_types::geometry::Point\">Point</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#58-63\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(point: <a class=\"struct\" href=\"geo_types/geometry/struct.Point.html\" title=\"struct geo_types::geometry::Point\">Point</a>&lt;T&gt;) -&gt; Self</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<Point<T>>","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Sub-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#173-183\">source</a><a href=\"#impl-Sub-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html\" title=\"trait core::ops::arith::Sub\">Sub</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"docblock\"><p>Subtract a coordinate from another.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>p = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.5</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>q = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.25</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>diff = p - q;\n\n<span class=\"macro\">assert_eq!</span>(diff.x, <span class=\"number\">0.25</span>);\n<span class=\"macro\">assert_eq!</span>(diff.y, <span class=\"number\">0.</span>);</code></pre></div>\n</div><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>The resulting type after applying the <code>-</code> operator.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.sub\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#177-182\">source</a><a href=\"#method.sub\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html#tymethod.sub\" class=\"fn\">sub</a>(self, rhs: Self) -&gt; Self</h4></section></summary><div class='docblock'>Performs the <code>-</code> operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html#tymethod.sub\">Read more</a></div></details></div></details>","Sub","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-PartialEq-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>self</code> and <code>other</code> values to be equal, and is used\nby <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#239\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>!=</code>. The default implementation is almost always\nsufficient, and should not be overridden without very good reason.</div></details></div></details>","PartialEq","geo_types::geometry::coord::Coordinate"],["<section id=\"impl-StructuralEq-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-StructuralEq-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.StructuralEq.html\" title=\"trait core::marker::StructuralEq\">StructuralEq</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section>","StructuralEq","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Zero-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#259-268\">source</a><a href=\"#impl-Zero-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.zero\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#261-263\">source</a><a href=\"#method.zero\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"num_traits/identities/trait.Zero.html#tymethod.zero\" class=\"fn\">zero</a>() -&gt; Self</h4></section></summary><div class='docblock'>Returns the additive identity element of <code>Self</code>, <code>0</code>. <a href=\"num_traits/identities/trait.Zero.html#tymethod.zero\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_zero\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#265-267\">source</a><a href=\"#method.is_zero\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"num_traits/identities/trait.Zero.html#tymethod.is_zero\" class=\"fn\">is_zero</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Returns <code>true</code> if <code>self</code> is equal to the additive identity.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_zero\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/num_traits/identities.rs.html#23\">source</a><a href=\"#method.set_zero\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"num_traits/identities/trait.Zero.html#method.set_zero\" class=\"fn\">set_zero</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Sets <code>self</code> to the additive identity element of <code>Self</code>, <code>0</code>.</div></details></div></details>","Zero","geo_types::geometry::coord::Coordinate"],["<section id=\"impl-Eq-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Eq-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section>","Eq","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Hash-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#impl-Hash-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> + <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#26\">source</a><a href=\"#method.hash\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#tymethod.hash\" class=\"fn\">hash</a>&lt;__H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>&gt;(&amp;self, state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut __H</a>)</h4></section></summary><div class='docblock'>Feeds this value into the given <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#tymethod.hash\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash_slice\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.3.0\">1.3.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/hash/mod.rs.html#238-240\">source</a></span><a href=\"#method.hash_slice\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#method.hash_slice\" class=\"fn\">hash_slice</a>&lt;H&gt;(data: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">[Self]</a>, state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut H</a>)<span class=\"where fmt-newline\">where\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</span></h4></section></summary><div class='docblock'>Feeds a slice of this type into the given <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#method.hash_slice\">Read more</a></div></details></div></details>","Hash","geo_types::geometry::coord::Coordinate"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Mul%3CT%3E-for-Coord%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#198-208\">source</a><a href=\"#impl-Mul%3CT%3E-for-Coord%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"geo_types/trait.CoordNum.html\" title=\"trait geo_types::CoordNum\">CoordNum</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;T&gt; for <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h3></section></summary><div class=\"docblock\"><p>Multiply coordinate wise by a scalar.</p>\n<h4 id=\"examples\"><a href=\"#examples\">Examples</a></h4>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>geo_types::coord;\n\n<span class=\"kw\">let </span>p = <span class=\"macro\">coord! </span>{ x: <span class=\"number\">1.25</span>, y: <span class=\"number\">2.5 </span>};\n<span class=\"kw\">let </span>q = p * <span class=\"number\">4.</span>;\n\n<span class=\"macro\">assert_eq!</span>(q.x, <span class=\"number\">5.0</span>);\n<span class=\"macro\">assert_eq!</span>(q.y, <span class=\"number\">10.0</span>);</code></pre></div>\n</div><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"geo_types/geometry/struct.Coord.html\" title=\"struct geo_types::geometry::Coord\">Coord</a>&lt;T&gt;</h4></section></summary><div class='docblock'>The resulting type after applying the <code>*</code> operator.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.mul\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/geo_types/geometry/coord.rs.html#202-207\">source</a><a href=\"#method.mul\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html#tymethod.mul\" class=\"fn\">mul</a>(self, rhs: T) -&gt; Self</h4></section></summary><div class='docblock'>Performs the <code>*</code> operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html#tymethod.mul\">Read more</a></div></details></div></details>","Mul<T>","geo_types::geometry::coord::Coordinate"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()