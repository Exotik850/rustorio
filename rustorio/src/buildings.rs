//! Buildings take inputs to produce outputs over time.
//!
//! To use a building, you must first build it which takes a number of resources.
//! Then you can add inputs to it using `add_input` or similar functions.
//! Once it has sufficient inputs, it will start producing outputs, which can be extracted using `take_output` or similar functions.
//!
//! When created, a building is set to a specific [`Recipe`](crate::recipes), which defines the inputs and outputs.
//! This can be changed using the `change_recipe` method, but only if the building is empty (no inputs or outputs).

use std::marker::PhantomData;

use crate::{
    Bundle, Resource, ResourceType, recipes::{AssemblerRecipe, FurnaceRecipe, Recipe}, sealed::Sealed, tick::Tick
};

pub trait Building<const I: usize, const O: usize, R: Recipe<I, O>>: Sealed {
    fn building_state(&self) -> &BuildingState<I, O, R>;
    fn building_state_mut(&mut self) -> &mut BuildingState<I, O, R>;
}

pub trait BuildingExt<const I: usize, const O: usize, R: Recipe<I, O>>: Building<I, O, R> {
    /// How much of each input resource is currently in the building.
    fn cur_inputs(&mut self, tick: &Tick) -> [u32; I] {
        self.building_state_mut().tick(tick);
        self.building_state().input_amounts
    }

    /// How much of each output resource is currently in the building.
    fn cur_outputs(&mut self, tick: &Tick) -> [u32; O] {
        self.building_state_mut().tick(tick);
        self.building_state().output_amounts
    }

    fn add_input<const INDEX: usize, const AMOUNT: u32>(&mut self, tick: &Tick, _bundle: Bundle<{ R::INPUTS[INDEX].resource() }, AMOUNT>) {
        self.building_state_mut().tick(tick);
        self.building_state_mut().input_amounts[INDEX] += AMOUNT;
    }

    fn take_output<const INDEX: usize, const AMOUNT: u32>(&mut self, tick: &Tick) -> Option<Bundle<{ R::OUTPUTS[INDEX].resource() }, AMOUNT>> {
        self.building_state_mut().tick(tick);
        if self.building_state().output_amounts[INDEX] >= AMOUNT {
            self.building_state_mut().output_amounts[INDEX] -= AMOUNT;
            Some(Bundle::new())
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct BuildingState<const I: usize, const O: usize, R> {
    input_amounts: [u32; I],
    output_amounts: [u32; O],
    tick: u64,
    start_time: Option<u64>,
    recipe: PhantomData<R>,
}

impl <const I: usize, const O: usize, R: Recipe<I, O>> BuildingState<I, O, R> {
    pub fn new(tick: &Tick) -> Self {
        Self {
            input_amounts: [0; I],
            output_amounts: [0; O],
            tick: tick.cur(),
            start_time: None,
            recipe: PhantomData,
        }
    }

    /// Replaces the current recipe with a new one.
    /// Returns the original building state if there are any inputs or outputs present.
    pub fn replace_recipe<const I2: usize, const O2: usize, R2: Recipe<I2, O2>>(self) -> Result<BuildingState<I2, O2, R2>, Self> {
        if self.input_amounts.iter().any(|&amt| amt > 0) || self.output_amounts.iter().any(|&amt| amt > 0) {
            Err(self)
        } else {
            Ok(BuildingState {
                input_amounts: [0; I2],
                output_amounts: [0; O2],
                tick: self.tick,
                start_time: None,
                recipe: PhantomData::<R2>,
            })
        }
    }

    pub fn tick(&mut self, tick: &Tick) {
        assert!(tick.cur() >= self.tick, "Tick must be non-decreasing");
        while self.tick < tick.cur() {
            self.tick += 1;
            if let Some(start_time) = self.start_time
                && self.tick >= start_time + R::TIME
                && self.input_amounts.iter().zip(R::INPUTS.iter()).all(|(&amt, slot)| amt >= slot.amount())
            {
                self.start_time = None;
                for (amt, slot) in self.input_amounts.iter_mut().zip(R::INPUTS.iter()) {
                    *amt -= slot.amount();
                }
                for (amt, slot) in self.output_amounts.iter_mut().zip(R::OUTPUTS.iter()) {
                    *amt += slot.amount();
                }
            }
            if self.start_time.is_none()
                && self.input_amounts.iter().zip(R::INPUTS.iter()).all(|(&amt, slot)| amt >= slot.amount())
            {
                self.start_time = Some(self.tick);
            }
        }
    }
}

/// The assembler is used for recipes that require different inputs to produce an output.
#[derive(Debug)]
pub struct Assembler<const I: usize, const O: usize, R> {
    building_state: BuildingState<I, O, R>,
}

/// Input [`Bundle`](Bundle) required to build an assembler.
type AssemblerIronInput = Bundle<{ ResourceType::Iron }, 15>;
/// Input [`Bundle`](Bundle) required to build an assembler.
type AssemblerCopperInput = Bundle<{ ResourceType::Copper }, 10>;

impl<const I: usize, const O: usize, R: AssemblerRecipe + Recipe<I, O>> Assembler<I, O, R> {
    /// Builds an assembler. Costs 15 iron and 10 copper.
    pub fn build(tick: &Tick, _iron: AssemblerIronInput, _copper: AssemblerCopperInput) -> Self {
        Self {
            building_state: BuildingState::new(tick),
        }
    }

    /// Changes the [`Recipe`](crate::recipes) of the assembler.
    /// Returns the original assembler if the assembler has inputs or outputs.
    pub fn change_recipe<const I2: usize, const O2: usize, R2: AssemblerRecipe + Recipe<I2, O2>>(self) -> Result<Assembler<I2, O2, R2>, Assembler<I, O, R>> {
        match self.building_state.replace_recipe::<I2, O2, R2>() {
            Ok(new_state) => Ok(Assembler {
                building_state: new_state,
            }),
            Err(original_state) => Err(Assembler {
                building_state: original_state,
            }),
        }
    }


}

/// The furnace is used to smelt ores into base resources.
#[derive(Debug)]
pub struct Furnace<R: FurnaceRecipe> {
    input_amount: u32,
    output_amount: u32,
    tick: u64,
    start_time: Option<u64>,
    recipe: PhantomData<R>,
}

/// Input [`Bundle`](Bundle) required to build a furnace.
type FurnaceIronInput = Bundle<{ ResourceType::Iron }, 10>;

impl<R: FurnaceRecipe> Furnace<R> {
    /// Builds a furnace. Costs 10 iron.
    pub fn build(tick: &Tick, _iron: FurnaceIronInput) -> Self {
        Self {
            input_amount: 0,
            output_amount: 0,
            tick: tick.cur(),
            start_time: None,
            recipe: PhantomData,
        }
    }

    /// Changes the [`Recipe`](crate::recipes) of the furnace.
    /// Returns the original furnace if the furnace has no inputs or outputs.
    pub fn change_recipe<R2: FurnaceRecipe>(self) -> Result<Furnace<R2>, Furnace<R>> {
        if self.input_amount > 0 || self.output_amount > 0 {
            Err(self)
        } else {
            Ok(Furnace {
                input_amount: 0,
                output_amount: 0,
                tick: self.tick,
                start_time: None,
                recipe: PhantomData::<R2>,
            })
        }
    }

    fn tick(&mut self, tick: &Tick) {
        assert!(tick.cur() >= self.tick, "Tick must be non-decreasing");
        while self.tick < tick.cur() {
            self.tick += 1;
            if let Some(start_time) = self.start_time
                && self.tick >= start_time + R::TIME
                && self.input_amount >= R::INPUT_AMOUNT
            {
                self.start_time = None;
                self.input_amount -= R::INPUT_AMOUNT;
                self.output_amount += R::OUTPUT_AMOUNT;
            }
            if self.start_time.is_none() && self.input_amount >= R::INPUT_AMOUNT {
                self.start_time = Some(self.tick);
            }
        }
    }

    /// How much of the input resource is currently in the furnace.
    pub fn cur_input(&mut self, tick: &Tick) -> u32 {
        self.tick(tick);
        self.input_amount
    }

    /// How much of the output resource is currently in the furnace.
    pub fn cur_output(&mut self, tick: &Tick) -> u32 {
        self.tick(tick);
        self.output_amount
    }

    /// Consumes a [`Bundle`](Bundle) and puts the contained resources into the furnace.
    pub fn add_input<const AMOUNT: u32>(&mut self, tick: &Tick, _ore: Bundle<{ R::INPUT }, AMOUNT>) {
        self.tick(tick);
        self.input_amount += AMOUNT;
    }

    /// Takes a specified amount of input resources from the furnace and puts it into a [`Bundle`](Bundle).
    pub fn take_input<const AMOUNT: u32>(&mut self, tick: &Tick) -> Option<Bundle<{ R::INPUT }, AMOUNT>> {
        self.tick(tick);
        if self.input_amount >= AMOUNT {
            self.input_amount -= AMOUNT;
            Some(Bundle::new())
        } else {
            None
        }
    }

    /// Takes all input resources currently in the furnace and puts it into a [`Resource`](Resource).
    pub fn empty_input(&mut self, tick: &Tick) -> Resource<{ R::INPUT }> {
        self.tick(tick);
        let amount = self.input_amount;
        self.input_amount = 0;
        Resource { amount }
    }

    /// Takes a specified amount of output resources from the furnace and puts it into a [`Bundle`](Bundle).
    pub fn take_output<const AMOUNT: u32>(&mut self, tick: &Tick) -> Option<Bundle<{ R::OUTPUT }, AMOUNT>> {
        self.tick(tick);
        if self.output_amount >= AMOUNT {
            self.output_amount -= AMOUNT;
            Some(Bundle::new())
        } else {
            None
        }
    }

    /// Takes all output resources currently in the furnace and puts it into a [`Resource`](Resource).
    pub fn empty_output(&mut self, tick: &Tick) -> Resource<{ R::OUTPUT }> {
        self.tick(tick);
        let amount = self.output_amount;
        self.output_amount = 0;
        Resource { amount }
    }
}
