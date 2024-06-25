(function () {
  "use strict";

  const url = document.body.classList.contains("debug")
    ? "http://127.0.0.1:8787/api/pet"
    : "https://cf.zyllian.workers.dev/api/pet";

  const petCounter = document.querySelector("#pet-counter");
  const count = petCounter.querySelector(".count");
  const petButton = petCounter.querySelector("button");

  (async function () {
    const r = await (await fetch(url)).json();
    count.innerText = r.count;
  })();

  petButton.addEventListener("click", async () => {
    petButton.disabled = true;
    petButton.innerText = "thanks! <3";
    const r = await (await fetch(url, { method: "post" })).json();
    if (r.count) {
      count.innerText = r.count;
    }
  });

  petCounter.style.display = "block";
})();
