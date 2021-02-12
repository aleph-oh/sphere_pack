//! Module used for computing sphere packing results given parameters.
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use serde::Serialize;
use spherical_cow::PackedVolume;
use thiserror::Error;

use crate::parsing;

/// A weighted distribution for selecting sphere radius.
#[derive(Debug)]
struct WeightedRadiusDistribution {
    /// Available radii
    choices: Vec<f64>,
    /// Distribution which can be drawn from to provide indices into choices
    dist: WeightedIndex<u8>,
}

impl Distribution<f64> for WeightedRadiusDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.choices[self.dist.sample(rng)]
    }
}

impl WeightedRadiusDistribution {
    /// Construct a new WeightedRadiusDistribution from an iterator `items`, where the first element
    /// of each tuple is a radius and the second element is the percentage (before division by 100)
    /// chance of that radius being drawn.
    fn new<I>(items: I) -> Self
    where
        I: IntoIterator<Item = (f64, u8)>,
    {
        let (choices, weights): (Vec<f64>, Vec<u8>) = items.into_iter().unzip();
        let dist = WeightedIndex::new(weights).unwrap();
        WeightedRadiusDistribution { choices, dist }
    }
}

/// A struct containing the output of one sphere packing simulation.
#[derive(Serialize)]
pub(crate) struct SimOutput {
    /// Packing efficiency fraction
    volume_fraction: f64,
    /// Surface area to volume ratio
    sa_to_vol: f64,
    sphere_count: usize,
}

#[derive(Debug, Error)]
/// An enumeration of all errors that can occur while running the simulation.
pub(crate) enum SimError {
    #[error("failed to pack shape")]
    FailedToPack(#[from] spherical_cow::errors::SphericalCowError),
}

/// Pack spheres into a cylinder, returning the result of this packing or an error to indicate
/// simulation failure.
pub(crate) fn pack(spheres: &parsing::Spheres) -> Result<SimOutput, SimError> {
    const TARGET_SPHERE_CT: f64 = 1000.;
    let sphere_volume = (spheres.avg_volume() * TARGET_SPHERE_CT) as f32;
    let cube_volume = (sphere_volume * 2.) as f32;
    let cube_side = cube_volume.cbrt();
    let container =
        spherical_cow::shapes::Cuboid::new(cube_side / 2., cube_side / 2., cube_side / 2.)
            .expect("Side lengths unexpectedly negative");
    let mut sizes = WeightedRadiusDistribution::new(spheres.iter().map(|s| (s.radius(), s.proportion())));
    let packed = PackedVolume::new(container, &mut sizes)?;
    Ok(SimOutput {
        volume_fraction: packed.volume_fraction() as f64,
        sa_to_vol: spheres.avg_volume() / spheres.avg_surface_area(),
        sphere_count: packed.spheres.len(),
    })
}
