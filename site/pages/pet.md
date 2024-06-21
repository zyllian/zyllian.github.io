---
title: pet
scripts: ["js/pet.js"]
styles: ["pet.css"]
---

<div id="pet-display">
	<p>--pet placeholder here--</p>
	<!-- TODO: make pets interactable -->
</div>

<div id="egg">
	<p>whoa! you just found an egg somewhere! maybe you should watch it and see what happens..</p>
</div>

<div id="pet-setup" class="hidden">
	<p>whoa! your egg just hatched into a new creature! what should you name it?</p>
	<input type="text" name="pet-name" max-length="50">
	<button>name it!</button>
</div>

<div id="debug-section">
	<button id="force-update">force update</button>
	<p>LS: <span name="ls"></span> A: <span name="a"></span> F: <span name="f"></span> B: <span name="b"></span> P: <span name="p"></span></p>
</div>
