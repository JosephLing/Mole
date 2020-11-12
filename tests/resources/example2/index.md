---
---

hello world

{{page | to_json | newline_to_br | strip_newlines }}

---------------------
{% assign a = 1 %}

{{a | plus: 1}}

----------------------

{{ page.title | append: page.title }}

