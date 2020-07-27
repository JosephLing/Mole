# Yes another static site generator
(hobby project practising rust)

Heavily inspired by Jekyll and  (https://github.com/cobalt-org/cobalt.rs)

A side project to able to learn Rust with a real world project. 


WIP:
- render support for jekyll in articles, currently works! 
  - even allows rendering other articles in an article kind of... and it's not really the most efficent method to do it. As we have to parse the `.pre_render().pre_render()` in order to evaluate it properly.
  - also the unit tests in parse are good but it is getting dangerously close to testing the mock not testing the code.... therefore integeration tests are needed!

  - need to have some kind of warning though about dealing with recursion:
    - potentailly don't allow it at all by seperating the global namespace up between posts and pages... 
    - see if it is possible to create an iterator that knows the context of use...?
    - leave it
    - ?????
    - (horrible idea) includes...

  The slight difficulty with the code being this simple/lazy while giving access for anything to know about anything else. Is that updating one means that any number of files could need to be updated. Therefore caching isn't quite possible for posts but maybe maybe for templates...


## todo:
- generating the correct output paths (need to add %20 to them as well maybe.... or handle that properly in the `serve` feature)
- javascript and images support

## quality of life:
- commands for:
  - clean
  - init

## advance
- serve up a mini http server
  - probably use `tiny_http`
- parallism of parsing files?
- tags and categories
- pagination maybe but not as important maybe....


potentailly could do this with jekyll or rust:
- sitemap
- rss



# Docs

## Sccs
We are using the grass library which is nearly feature complete but missing @use and a few other sass rules. Their next release will have some improvements to @imports though and other things :)

## varaibles

These maybe a little out-dated check in `src/parser.rs` for the latests....
```
{
    global:{
        articles: []
    },
    config: {
        title,
        description,
        tags,
        categories,
        visible,
        layout
    },
    url
}
```

e.g. {{config.title}}

You can do:
```
{% for art in global.articles %}
  {{art.content}}
{% endfor %}
```
To cause a recursive pattern of posts but that will include the current post. Is currently only fixed to do 2 levels of recursion so it would be best to wrap in an if statement `{{% if art.title != config.tilte %}}`.

## command line

`mole build`

- build: runs from the current directory 
  - `_output/` to the path
  - `_articles/` for all the posts (*.html)
  - `_include/` and `_layout/` will be all the liquid includes (*.md)
  - `_css/` for sass (*.sass)


Example:
`mole build tests\resources\example1`


### serve

`mole build --serve true`

Spins up a mini web server on localhost at port 4000 so `127.0.0.1:4000` (hard coded).

### watch
`mole build --watch true`
Rebuilds every time there is a change, this feature though is still WIP.


## render pipeline
- includes and layouts to generate templates
- then to get all the posts and we check for the layout to make sure it actually exists
- build the varaible 
  - global contains all the posts/articles
- render all the posts
  - include 'default' or if base_layout is defined in the file 


(nothing else is implemented)


## articles

required:
```
---
layout:page
title: give me all the dreamies
description: hello world
---
asdfasd
```

potentail:
- `base_layout`
- `permalink`
- `visible`


