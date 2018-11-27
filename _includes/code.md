<a style="position: absolute; left: 80%" target="_blank" href="#">
  Full example on Playground
  (TODO: Position this link nicely)
</a>

```rust
{% include_relative _async-await/snippets/{{include.code}} %}
```

<script type="text/rust">{% include_relative _async-await/playgrounds/{{include.code}} %}</script>
<script type="text/javascript">
let me = document.currentScript
let playground = me.previousElementSibling
let link = playground.previousElementSibling.previousElementSibling.children[0]
link.href = `https://play.rust-lang.org/?version=nightly&edition=2018&code=${encodeURIComponent(playground.text)}`
</script>
