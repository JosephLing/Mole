---
layout:page
title: content & foo
description: hello world
tags: programming, testing, food
---

- d
- b
- c

meta data {{page.config.description}} base layout: {{page.config.layout}}
{{page.config.tags}}


# page description

{% for art in global.articles %}
   {% if art.content != page.content %}
<div>
    <h1>{{art.config.title}}</h1>
    <span>{{art.content}}</span>
</div>
        
   {% endif %}
{% endfor %}