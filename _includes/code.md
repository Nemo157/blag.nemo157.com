{% capture snippet %}
{% include_relative _async-await/snippets/{{include.code}} %}
{% endcapture %}

{% capture playground %}
{% include_relative _async-await/playgrounds/{{include.code}} %}
{% endcapture %}

```rust
{{snippet | lstrip | rstrip}}
```

<a class="icon play" style="display: none" target="_blank" href="#" title="Run on playground">{% include icons/play.svg %}</a>

<script type="text/rust">{{playground | lstrip | rstrip}}</script>
<script type="text/javascript">{% include code.js %}</script>
