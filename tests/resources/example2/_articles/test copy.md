---
layout:page
title: contents page
description: hello world
---
# links md:
{% for art in global.articles %}
    - [{{art.title}}]({{art.url}})
{% endfor %}



# links html:
{% for art in global.articles %}
<a href="{{art.url}}">{{art.title}}</a>
{% endfor %}

{% capture my_variable %}I am being captured.{% endcapture %}