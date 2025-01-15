---
title: introducing webdog
timestamp: "2024-11-13T23:02:36.179943751Z"
tags:
  - webdog
  - meta
  - dev
cdn_file: webdog.jpg
header_image_alt: placeholder
desc: my static site generator, now available for anyone :3
---

# [webdog.](https://webdog.zyl.gay)

if you've been to my site recently, you may have noticed the "powered by webdog" text in the footer. if not, then surprise! i've been working on upgrading the tool i built for this site to make it easier for anyone to use.

if you have [rust](https://rust-lang.org) installed, you can install webdog like this:

```sh
cargo install webdog
```

and that's it! now you can use webdog to build a site of your own! if i were creating zyl.gay again today, i'd run this command to get started:

```sh
webdog create --site ./zyllian.github.io https://zyl.gay/ "zyl is gay" --cdn-url https://i.zyl.gay/
```

from here, you could consult the [webdog documentation](https://webdog.zyl.gay/docs/) for more info on how to build your site.
