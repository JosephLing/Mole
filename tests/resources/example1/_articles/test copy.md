---
layout:page
title: contents page
description: hello world
---
# links md:
{% for art in global.articles %}
    - [{{art.config.title}}]({{art.url}})
{% endfor %}


{{asfsdfs}}

# links html:
{% for art in global.foobar %}
<a href="{{art.url}}">{{art.config.title}}</a>
{% endfor %}