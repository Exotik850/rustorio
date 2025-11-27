//! A recipe is a way of turning resources into other resources.
//! A specific recipe specifies the input and output resources, as well as the time it takes to complete the recipe.

use std::{fmt::Debug, num::NonZero};

use crate::{ResourceType, sealed::Sealed};

#[derive(Debug, Clone, Copy)]
pub struct RecipeSlot {
    amount: u32,
    resource: ResourceType,
}

impl RecipeSlot {
    pub const fn amount(&self) -> u32 {
        self.amount
    }

    pub const fn resource(&self) -> ResourceType {
        self.resource
    }
}

pub trait Recipe<const INPUTS: usize, const OUTPUTS: usize>: Debug + Sealed {
    const INPUTS: [RecipeSlot; INPUTS];
    const OUTPUTS: [RecipeSlot; OUTPUTS];
    const TIME: NonZero<u32>;

    fn input(index: usize) -> Option<RecipeSlot> {
        Self::INPUTS.get(index).copied()
    }

    fn output(index: usize) -> Option<RecipeSlot> {
        Self::OUTPUTS.get(index).copied()
    }
}

pub trait SimpleRecipe: Recipe<1, 1> {
    const INPUT: RecipeSlot = Self::INPUTS[0];
    const OUTPUT: RecipeSlot = Self::OUTPUTS[0];
}

impl<R: Recipe<1, 1>> SimpleRecipe for R {}

pub trait DualInputRecipe: Recipe<2, 1> {
    const INPUT1: RecipeSlot = Self::INPUTS[0];
    const INPUT2: RecipeSlot = Self::INPUTS[1];
    const OUTPUT: RecipeSlot = Self::OUTPUTS[0];
}

impl<R: Recipe<2, 1>> DualInputRecipe for R {}

/// Any recipe that implements this trait can be used in an [`Assembler`](crate::buildings::Assembler).
pub trait AssemblerRecipe: Debug + Sealed {}

/// The recipe you need to win! An [`Assembler`](crate::buildings::Assembler) recipe that creates points. Converts 4 iron and 4 copper into 1 point resource. Takes 20 ticks.
#[derive(Debug)]
pub struct PointRecipe;

impl Sealed for PointRecipe {}
impl AssemblerRecipe for PointRecipe {}

impl Recipe<2, 1> for PointRecipe {
    const INPUTS: [RecipeSlot; 2] = [
        RecipeSlot {
            amount: 4,
            resource: ResourceType::Iron,
        },
        RecipeSlot {
            amount: 4,
            resource: ResourceType::Copper,
        },
    ];
    const OUTPUTS: [RecipeSlot; 1] = [RecipeSlot {
        amount: 1,
        resource: ResourceType::Point,
    }];
    const TIME: NonZero<u32> = NonZero::new(20).unwrap();
}

/// Any recipe that implements this trait can be used in a [`Furnace`](crate::buildings::Furnace).
pub trait FurnaceRecipe: Debug + Sealed {}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts iron ore into iron. Converts 2 iron ore into 1 iron. Takes 10 ticks.
#[derive(Debug)]
pub struct IronSmelting;

impl Sealed for IronSmelting {}

impl FurnaceRecipe for IronSmelting {}

impl Recipe<1, 1> for IronSmelting {
    const INPUTS: [RecipeSlot; 1] = [RecipeSlot {
        amount: 2,
        resource: ResourceType::IronOre,
    }];
    const OUTPUTS: [RecipeSlot; 1] = [RecipeSlot {
        amount: 1,
        resource: ResourceType::Iron,
    }];
    const TIME: NonZero<u32> = NonZero::new(10).unwrap();
}

/// A [`Furnace`](crate::buildings::Furnace) recipe that smelts copper ore into copper. Converts 2 copper ore into 1 copper. Takes 10 ticks.
#[derive(Debug)]
pub struct CopperSmelting;

impl Sealed for CopperSmelting {}

impl FurnaceRecipe for CopperSmelting {
    // const INPUT: ResourceType = ResourceType::CopperOre;
    // const INPUT_AMOUNT: u32 = 2;
    // const OUTPUT: ResourceType = ResourceType::Copper;
    // const OUTPUT_AMOUNT: u32 = 1;
    // const TIME: u64 = 10;
}

impl Recipe<1, 1> for CopperSmelting {
    const INPUTS: [RecipeSlot; 1] = [RecipeSlot {
        amount: 2,
        resource: ResourceType::CopperOre,
    }];
    const OUTPUTS: [RecipeSlot; 1] = [RecipeSlot {
        amount: 1,
        resource: ResourceType::Copper,
    }];
    const TIME: NonZero<u32> = NonZero::new(10).unwrap();
}
