(function () {
  "use strict";

  const click = document.querySelector("#click");
  const petsCounter = click.querySelector("#pets");
  const petsPerSecondCounter = click.querySelector("#pets-per-second");
  const barksCounter = click.querySelector("#barks");
  const barksPerSecondCounter = click.querySelector("#barks-per-second");
  const kissesCounter = click.querySelector("#kisses");
  const kissesPerSecondCounter = click.querySelector("#kisses-per-second");
  const barker = click.querySelector("#barker");
  const toolsEl = click.querySelector(".tools");

  const toolPriceFactor = 0.1;
  const upgradePriceFactor = 0.2;
  const upgradeProductionFactor = 1.1;

  const toolData = {
    hand: {
      priceIn: "barks",
      basePrice: 10,
      petsPerSecond: 0.5,
    },
    puppy: {
      priceIn: "pets",
      basePrice: 5,
      barksPerSecond: 0.5,
    },
    foodBowl: {
      priceIn: "barks",
      basePrice: 50,
      barksPerSecond: 1.3,
    },
    kisser: {
      priceIn: "pets",
      basePrice: 500,
      kissesPerSecond: 0.25,
    },
  };

  let barks = 0;
  let pets = 0;
  let kisses = 0;
  let tools = {};

  let petsPerSecond = 0;
  let barksPerSecond = 0;
  let kissesPerSecond = 0;

  function calcPrice(base, count) {
    return Math.floor(base ** (1 + toolPriceFactor * count));
  }

  function calcUpgradePrice(base, count) {
    return Math.floor((base * 2) ** (1 + upgradePriceFactor * count));
  }

  const getValue = (name) => {
    if (name === "pets") {
      return pets;
    } else if (name === "barks") {
      return barks;
    } else if (name === "kisses") {
      return kisses;
    } else if (name === "petsPerSecond") {
      return petsPerSecond;
    } else if (name === "barksPerSecond") {
      return barksPerSecond;
    } else if (name === "kissesPerSecond") {
      return kissesPerSecond;
    }
  };

  const setValue = (name, value) => {
    if (name === "pets") {
      pets = value;
    } else if (name === "barks") {
      barks = value;
    } else if (name === "kisses") {
      kisses = value;
    } else if (name === "petsPerSecond") {
      petsPerSecond = value;
    } else if (name === "barksPerSecond") {
      barksPerSecond = value;
    } else if (name === "kissesPerSecond") {
      kissesPerSecond = value;
    }
  };

  const updatePerSecondValues = () => {
    let pets = 0;
    let barks = 0;
    let kisses = 0;

    for (const [id, tool] of Object.entries(tools)) {
      pets +=
        (toolData[id].petsPerSecond || 0) *
        tool.count *
        tool.upgrades *
        upgradeProductionFactor;
      barks +=
        (toolData[id].barksPerSecond || 0) *
        tool.count *
        tool.upgrades *
        upgradeProductionFactor;
      kisses +=
        (toolData[id].kissesPerSecond || 0) *
        tool.count *
        tool.upgrades *
        upgradeProductionFactor;
    }

    petsPerSecond = pets;
    barksPerSecond = barks;
    kissesPerSecond = kisses;
  };

  const updateDisplay = () => {
    petsCounter.innerText = pets;
    petsPerSecondCounter.innerText = petsPerSecond.toFixed(2);
    barksCounter.innerText = barks;
    barksPerSecondCounter.innerText = barksPerSecond.toFixed(2);
    kissesCounter.innerText = kisses;
    kissesPerSecondCounter.innerText = kissesPerSecond.toFixed(2);
  };

  for (const el of toolsEl.querySelectorAll(".tool")) {
    const id = el.getAttribute("data-tool");
    if (id) {
      const data = toolData[id];
      if (data) {
        const toolInfo = {
          count: 0,
          upgrades: 1,
        };
        tools[id] = toolInfo;

        const count = el.querySelector(".count");
        const level = el.querySelector(".level");
        const buy = el.querySelector(".buy");
        const upgrade = el.querySelector(".upgrade");

        const updateText = () => {
          count.innerText = toolInfo.count;
          level.innerText = toolInfo.upgrades;
          const price = calcPrice(data.basePrice, toolInfo.count);
          const upgradePrice = calcUpgradePrice(
            data.basePrice,
            toolInfo.upgrades
          );
          buy.innerText = `buy - ${price} ${data.priceIn}`;
          upgrade.innerText = `upgrade - ${upgradePrice} kisses`;
        };
        updateText();

        buy.addEventListener("click", () => {
          const price = calcPrice(data.basePrice, toolInfo.count);
          const v = getValue(data.priceIn);
          if (v >= price) {
            setValue(data.priceIn, v - price);
            toolInfo.count += 1;
            updatePerSecondValues();
            updateText();
            updateDisplay();
          }
        });

        upgrade.addEventListener("click", () => {
          const price = calcUpgradePrice(data.basePrice, toolInfo.upgrades);
          if (kisses >= price) {
            kisses -= price;
            toolInfo.upgrades += 1;
            updatePerSecondValues();
            updateText();
            updateDisplay();
          }
        });
      }
    }
  }

  barker.addEventListener("click", () => {
    barks += 1;
    updateDisplay();
  });

  let lastUpdate = 0;
  let petsQueued = 0;
  let barksQueued = 0;
  let kissesQueued = 0;

  const checkQueue = (name, queued) => {
    const perSecond = getValue(`${name}PerSecond`);
    if (perSecond > 0) {
      const amount = 1000 / perSecond;
      const toAdd = Math.floor(queued / amount);
      setValue(name, getValue(name) + toAdd);
      updateDisplay();
      queued -= toAdd * amount;
    } else {
      queued = 0;
    }
    return queued;
  };

  const update = (ts) => {
    requestAnimationFrame(update);

    const diff = ts - lastUpdate;
    petsQueued += diff;
    barksQueued += diff;
    kissesQueued += diff;

    petsQueued = checkQueue("pets", petsQueued);
    barksQueued = checkQueue("barks", barksQueued);
    kissesQueued = checkQueue("kisses", kissesQueued);

    lastUpdate = ts;
  };

  requestAnimationFrame(update);
})();
