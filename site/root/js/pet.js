(function () {
  "use strict";

  /** the current pet version.. */
  const CURRENT_PET_VERSION = 1;
  /** the max food a pet will eat */
  const MAX_FOOD = 100;
  /** the amount of time it takes for a pet to have to GO */
  const POTTY_TIME = 100;
  /** how fast a pet's food value decays */
  const FOOD_DECAY = 5;
  /** the rate at which a pet ages */
  const AGING_RATE = 1;
  /** how fast a pet's potty need decays */
  const POTTY_DECAY = 5;

  /** life stage for an egg */
  const LIFE_STAGE_EGG = 1;
  /** life stage for a pup */
  const LIFE_STAGE_PUP = 2;
  /** life stage for an adult */
  const LIFE_STAGE_ADULT = 3;
  /** life stage for an elder pet */
  const LIFE_STAGE_ELDER = 4;
  /** the time it takes for a pet to grow past the egg phase */
  const EGG_TIME = 2;
  /** the time it takes for a pet to grow past the pup phase */
  const PUP_TIME = 300;
  /** the time it takes for a pet to grow past the adult phase */
  const ADULT_TIME = 900;
  /** the time it takes for a pet to grow past the elder phase */
  const ELDER_TIME = 300;

  const eggDiv = document.querySelector("div#egg");
  const petSetup = document.querySelector("div#pet-setup");
  const name = petSetup.querySelector("input[name=pet-name]");
  const nameItButton = petSetup.querySelector("button");
  const debug = document.querySelector("div#debug-section");
  const debugLifeStage = debug.querySelector("span[name=ls]");
  const debugAge = debug.querySelector("span[name=a]");
  const debugFood = debug.querySelector("span[name=f]");
  const debugBehavior = debug.querySelector("span[name=b]");
  const debugPotty = debug.querySelector("span[name=p]");
  const forceUpdateButton = debug.querySelector("button#force-update");

  /**
   * class containing information about a pet
   */
  class Pet {
    /** current pet version */
    version = CURRENT_PET_VERSION;
    /** whether the pet can die or not */
    canDie = false;
    /** whether the pet is alive or dead */
    alive = true;
    /** whether the pet simulation is paused */
    paused = false;
    /** the pet's current life stage */
    lifeStage = LIFE_STAGE_EGG;
    /** the pet's name */
    name = "";
    /** how much food the pet has stored */
    food = MAX_FOOD;
    /** the pet's age */
    age = 0;
    /** the pet's behavior score */
    behavior = 0;
    /** how long until the pet needs to go potty */
    pottyTimer = POTTY_TIME;
    /** the time the pet was last updated */
    lastUpdate = Date.now();
    /** the time the egg was found */
    eggFound = Date.now();
    /** the time the egg hatched */
    hatched = Date.now();

    /**
     * updates a pet
     */
    update() {
      if (!this.alive || this.paused) {
        return;
      }
      console.log("update");

      this.age += AGING_RATE;

      if (this.lifeStage !== LIFE_STAGE_EGG) {
        this.food -= FOOD_DECAY;
        this.pottyTimer -= POTTY_DECAY;

        if (this.food < 0) {
          if (this.canDie) {
            // TODO: pet dies
          } else {
            this.food = 0;
          }
        }

        if (this.pottyTimer < 0) {
          this.goPotty();
        }
      }

      if (this.lifeStage === LIFE_STAGE_EGG && this.age >= EGG_TIME) {
        this.paused = true;
        this.lifeStage = LIFE_STAGE_PUP;
        this.age = 0;
      } else if (this.lifeStage === LIFE_STAGE_PUP && this.age >= PUP_TIME) {
        this.paused = true;
        this.lifeStage = LIFE_STAGE_ADULT;
        this.age = 0;
      } else if (
        this.lifeStage === LIFE_STAGE_ADULT &&
        this.age >= ADULT_TIME
      ) {
        this.paused = true;
        this.lifeStage = LIFE_STAGE_ELDER;
        this.age = 0;
      } else if (
        this.lifeStage === LIFE_STAGE_ELDER &&
        this.age >= ELDER_TIME
      ) {
        this.paused = true;
        this.alive = false;
        // TODO: DEATH
      }
      this.updateDom();
    }

    /**
     * updates the html dom
     */
    updateDom() {
      eggDiv.classList.add("hidden");
      petSetup.classList.add("hidden");

      if (this.lifeStage === LIFE_STAGE_EGG) {
        eggDiv.classList.remove("hidden");
      } else if (this.lifeStage === LIFE_STAGE_PUP) {
        if (this.paused && this.name === "") {
          petSetup.classList.remove("hidden");
        }
      } else if (this.lifeStage === LIFE_STAGE_ADULT) {
      } else if (this.lifeStage === LIFE_STAGE_ELDER) {
      }

      debugLifeStage.innerText = this.lifeStage;
      debugAge.innerText = this.age;
      debugFood.innerText = this.food;
      debugBehavior.innerText = this.behavior;
      debugPotty.innerText = this.pottyTimer;
    }

    /**
     * feeds the pet
     * @param {number} amount the amount to feed the pet by
     */
    feed(amount) {
      if (this.food > MAX_FOOD) {
        return;
      }
      this.food += amount;
    }

    /**
     * makes the pet go potty
     */
    goPotty() {
      if (this.behavior > 15) {
        // go potty properly
      } else {
        // make a mess of that shit
      }
      this.pottyTimer = POTTY_TIME;
    }
  }

  let pet = new Pet();
  pet.updateDom();

  nameItButton.addEventListener("click", () => {
    const newName = name.value;
    console.log(newName);
    if (newName.trim().length === 0) {
      return;
    }
    pet.name = newName;
    pet.paused = false;
    pet.updateDom();
  });

  const update = () => {
    pet.update();
  };

  // update the pet every 30 seconds?
  setInterval(update, 30000);

  forceUpdateButton.addEventListener("click", update);

  console.log(pet);
})();
