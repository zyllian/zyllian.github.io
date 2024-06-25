(function () {
  "use strict";

  const DEBOUNCE_TIMER = 1500;

  const url = document.body.classList.contains("debug")
    ? "http://127.0.0.1:8787/api/pet"
    : "https://cf.zyllian.workers.dev/api/pet";
  console.log(url);

  const petCounter = document.querySelector("#pet-counter");
  const count = petCounter.querySelector(".count");
  const petButton = petCounter.querySelector("button");

  (async function () {
    const r = await (await fetch(url)).json();
    count.innerText = r.count;
  })();

  petButton.addEventListener("click", async () => {
    petButton.disabled = true;
    setTimeout(() => (petButton.disabled = false), DEBOUNCE_TIMER);
    const r = await (await fetch(url, { method: "post" })).json();
    if (r.count) {
      count.innerText = r.count;
    }
  });

  petCounter.style.display = "block";
})();
