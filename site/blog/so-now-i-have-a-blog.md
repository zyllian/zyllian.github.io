---
title: so now i have a blog
timestamp: 2023-06-09T16:00:00.00Z
tags: [meta, technical]
desc: I added a blog to my site.
header_image_file: cat.jpeg # TODO: placeholder image
header_image_alt: placeholder
draft: true
---

So now I have a blog on my site. I don't really have any plans to post here regularly, but idk maybe that'll change in the future.

That's pretty much it as far as the non-technical side of things goes.

## The Technical Side of Things

I haven't really written anything about how my site works before, so this is also going to contain some general information about the site as a whole.

`zyl.gay` is a static website built with a custom static site builder I built for it. It started by taking Markdown pages and rendering them on top of the appropriate template.

When I added the [images section](/images/) to the site I added the first abstraction on top of this: YAML files with the relevant metadata for the image (including a short but unstyled description) which then get rendered not only into pages for the individual pages, but also a paginated display for all the images _and_ a method to view images by tag.

To get blogs working I modified the image page code to be generic over provided resource types, so really the images and the blog posts are rendered the same way, just with different configurations.
