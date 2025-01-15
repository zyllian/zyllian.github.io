---
title: i built a minecraft classic server
timestamp: 2024-07-08T10:37:00.00Z
tags: [minecraft, minecraft classic, dev]
desc: i got bored and built a minecraft classic server
cdn_file: 2024/07/classic2.png
header_image_alt: Screenshot of my Minecraft classic server showing two players standing in front of a stack of blocks.
---

a while ago i was looking for something to do and my brain decided i should.. build a complete reimplementation of the minecraft server, naturally. i knew i'd never get anywhere close to finishing that project before losing steam so instead, i redirected this inspiration into [a minecraft classic server](https://github.com/zyllian/classics)!

<wd-partial t="blog/blog-image.tera" class="float-left w50" src="cdn$2024/07/classic1.png">
	at one point while changing my world format, worlds started getting sent to clients diagonally! don't remember what caused this, though.
</wd-partial>
<!-- <md class="float-left w50" type="blog-image" src="cdn$2024/07/classic1.png" content="at one point while changing my world format, worlds started getting sent to clients diagonally! don't remember what caused this, though."></md> -->

minecraft classic has [fairly simple networking](https://wiki.vg/Classic_Protocol) as it turns out, so building a server which vanilla classic clients can connect to isn't too bad if you know what you're doing, and with no need to stay true to the vanilla map format, you can simplify things even further!

minecraft classic actually still has a decently large community around it, and what i believe are the most popular clients/servers all implement [an extended networking protocol](https://wiki.vg/Classic_Protocol_Extension) to add extra features that vanilla classic doesn't support.

my implementation lacks basically any kind of anti-cheat and is lacking most of the community protocol extensions, but it's still pretty cool to have built it! i love working on random projects and it's nice to see one which is actually useable, even if there's [a lot to add](https://zyllian/classics/issues)!

<wd-partial t="blog/blog-image.tera" src="cdn$2024/07/classic3.png">
	screenshot of me stress testing how many players i could connect to the server :3
</wd-partial>
<!-- <md type="blog-image" src="cdn$2024/07/classic3.png" content="screenshot of me stress testing how many players i could connect to the server :3"></md> -->
