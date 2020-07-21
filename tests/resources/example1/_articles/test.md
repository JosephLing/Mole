---
layout:page
title: give me all the dreamies
description: hello world
---

- d
- b
- c

meta data {{page.config.description}} base layout: {{page.config.layout}}



# page description

{% for art in global.articles %}
   {% if art.content != page.content %}
<div>
    <h1>{{art.config.title}}</h1>
    <span>{{art.content}}</span>
</div>
        
   {% endif %}
{% endfor %}
