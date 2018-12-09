{% capture snippet %}
{% include_relative _async-await/snippets/{{include.code}} %}
{% endcapture %}

{% capture playground %}
{% include_relative _async-await/playgrounds/{{include.code}} %}
{% endcapture %}

```rust
{{snippet | lstrip | rstrip}}
```

<a class="icon play" target="_blank" href="#" title="Run on playground">{% include icons/play.svg %}</a>

<script type="text/rust">{{playground | lstrip | rstrip}}</script>
<script type="text/javascript">
(() => {
  let me = document.currentScript
  let playground = me.previousElementSibling
  let linkContainer = playground.previousElementSibling
  let link = linkContainer.children[0]
  let snippet = linkContainer.previousElementSibling.children[0].children[0]
  link.href = `https://play.rust-lang.org/?version=nightly&edition=2018&code=${encodeURIComponent(playground.text)}`
  snippet.style = 'position: relative;'
  snippet.prepend(link)
})()
</script>
