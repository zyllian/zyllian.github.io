---
title: pet
scripts: ["js/pet.js"]
styles: ["pet.css"]
---

<div id="pet">
<noscript><p>javascript is required for the pet game!!</p></noscript>
<div id="pet-display">
	<h2 class="pet-name"></h2>
	<div class="the-pet"></div>
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
		<li>the game (currently) doesn't simulate while the page is unloaded, so make sure to keep the page loaded for your pet to exist!</li>
		<li>make sure to keep your pet clean!!</li>
		<li>if your pet is turning grey, make sure you're giving them the attention they need!! pet's deserve happiness too :(</li>
	</ul>
</details>
