---
title: pet
scripts: ["js/pet.js"]
styles: ["pet.css"]
embed:
  title: raise a pet
  site_name: test
  description: come raise a cool pet! you could get a square. or a circle. or a triangle. who knows?
---

<div id="pet">
<noscript><p>javascript is required for the pet game!!</p></noscript>
<div id="pet-display">
	<h2 class="pet-name"></h2>
	<div class="the-pet"></div>
	<div class="status">
		<p name="hungry" class="hidden"><span class="pet-name"></span> looks hungry..</p>
		<p name="starving" class="hidden"><span class="pet-name"></span> is starving!! you need to feed them!!</p>
		<p name="unhappy" class="hidden"><span class="pet-name"></span> looks at you with wide eyes..</p>
		<p name="messy-1" class="hidden"><span class="pet-name"></span> has left a bit of a mess! time to clean!</p>
		<p name="messy-2" class="hidden">there's even more mess in here.. shouldn't you clean it for <span class="pet-name"></span>?</p>
		<p name="messy-3" class="hidden">what a mess!! <span class="pet-name"></span> can't be happy.. you've gotta clean in here</p>
	</div>
</div>

<div id="egg">
	<p>whoa! you just found a weird egg! maybe you should watch it and see what happens..</p>
</div>

<div id="adult-info" class="hidden">
	<p><span class="pet-name"></span> has grown up to be an adult!! what will they do with their life now....</p>
	<button class="advance">okay!</button>
</div>

<div id="elder-info" class="hidden">
	<p>oh? <span class="pet-name"></span> has aged and is now an elder creature! they may not have much left in their life.... hopefully it's been a good life!</p>
	<button class="advance">hopefully!!</button>
</div>

<div id="passed-away-info" class="hidden">
	<p>oh... <span class="pet-name"></span> has finally gone and kicked the bucket. its story comes to an end....</p>
	<button>but what's this egg lying here about..?</button>
</div>

<form id="pet-setup" class="hidden">
	<p>whoa! your egg just hatched into a new creature! what should you name it?</p>
		<input type="text" name="pet-name" min-length="3" max-length="50">
		<button type="submit">name it!</button>
</form>

<div id="pet-actions">
	<div name="hatched-actions" class="hidden">
		<button name="feed">feed</button>
		<button name="pet">pet</button>
		<button name="clean">clean</button>
	</div>
	<button name="pause">pause</button>
</div>

<div id="debug-section" class="hidden">
	<button id="force-update">force update</button> <button id="reset">reset</button>
	<p>LS: <span name="ls"></span> A: <span name="a"></span> F: <span name="f"></span> B: <span name="b"></span> P: <span name="p"></span> MC: <span name="mc"></span> H: <span name="h"></span></p>
</div>
</div>

<details>
	<summary>tips!!</summary>
	<ul>
		<li>pets need to be fed about once every eight hours!</li>
		<li>your pet still exists while the page is unloaded or your computer is off! pause if you need to leave them be for a while!</li>
		<li>make sure to keep your pet clean!!</li>
		<li>if your pet is turning grey, make sure you're giving them the attention they need!! pet's deserve happiness too :(</li>
		<li>if you take good enough care of your pet they'll stop going potty on the floor!</li>
	</ul>
</details>

<details open>
	<summary>changelog</summary>
	<details>
		<summary>07/08/2024</summary>
		<ul>
			<li>slow pet food/happiness decay</li>
		</ul>
	</details>
	<details>
		<summary>07/04/2024</summary>
		<ul>
			<li>oops pets couldn't age past puppies..</li>
		</ul>
	</details>
	<details>
		<summary>06/25/2024</summary>
		<ul>
			<li>pets now are simulated even if the page is unloaded</li>
			<li>when pets are unhappy &lt;redacted&gt;<!-- their hidden behavior stat drops --></li>
		</p>
	</details>
</details>
