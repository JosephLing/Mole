# Yes another static site generator
(hobby project practising rust)

Heavily inspired by Jekyll and  (https://github.com/cobalt-org/cobalt.rs)

A side project to able to learn Rust with a real world project. 

## todo:
- generating the correct output paths
- javascript and images support
- render support for jekyll in articles (currently test for it so it will be failing until fixed)

## quality of life:
- commands for:
  - clean
  - init
  - serve


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

## command line

`mole build`

- build: runs from the current directory 
  - `_output/` to the path
  - `_source/` and `_articles/` for all the posts (*.html)
  - `_include/` and `_layout/` will be all the liquid includes (*.md)
  - `_css/` for sass (*.sass)

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


