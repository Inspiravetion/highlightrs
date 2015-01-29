highlightrs
===========

"A command line utility to turn arbitrary rust code into syntax highlighted html"

Currently based on `rustdoc::html::highlight::highlight` but transitioning to using `syntax::parse::parser::Parser` to get an AST to then highlight.

As of now,

```bash
$> highlightrs 'let a = "b";'
```
produces
```html
<pre class='rust '>
<span class='kw'>let</span> <span class='ident'>a</span> <span class='op'>=</span> <span class='string'>&quot;b&quot;</span>;
</pre>
```

# Usage
```
Options:
    -i --inputfile FILE use a file for the input
    -o --outfile FILE   use a file for the output
    -h --help           print this help menu
```
