//! Module used for computing sphere packing results given parameters.

use std::f32::consts::PI;

use nalgebra::Point3;
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use serde::Serialize;
use spherical_cow::shapes::Sphere;
use spherical_cow::PackedVolume;
use thiserror::Error;

use crate::parsing;

/// A cylindrical shape which can be used as a container for spheres in packing.
struct Cylinder {
    /// Radius of cylinder container
    radius: f32,
    /// Height of cylinder container
    height: f32,
    /// Location of the center of cylinder container
    center: Point3<f32>,
}

impl spherical_cow::Container for Cylinder {
    fn contains(&self, sphere: &Sphere) -> bool {
        self.radius < sphere.radius
            && sphere.center.z + sphere.radius < self.center.z + self.height / 2.0
            && sphere.center.z - sphere.radius < self.center.z - self.height / 2.0
            && (sphere.center.x - self.center.x).powi(2)
                + (sphere.center.y - self.center.y).powi(2)
                + (sphere.center.z - self.center.z).powi(2)
                <= (self.radius - sphere.radius).powi(2)
    }

    fn volume(&self) -> f32 {
        PI * self.radius.powi(2) * self.height
    }
}

impl Cylinder {
    /// Construct a new Cylinder with the given parameters.
    /// Panics if radius or height are less than 0.
    fn new(bottom: Point3<f32>, radius: f32, height: f32) -> Self {
        assert!(radius > 0.0);
        assert!(height > 0.0);
        Cylinder {
            radius,
            height,
            center: Point3::new(bottom.x, bottom.y, bottom.z + height),
        }
    }
}

/// A weighted distribution for selecting sphere radius.
struct WeightedRadiusDist {
    /// Available radii
    choices: Vec<f64>,
    /// Distribution which can be drawn from to provide indices into choices
    dist: WeightedIndex<u8>,
}

impl Distribution<f64> for WeightedRadiusDist {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.choices[self.dist.sample(rng)]
    }
}

impl WeightedRadiusDist {
    /// Construct a new WeightedRadiusDist from an iterator `items`, where the first element is a
    /// radius and the second element is the percentage (before division by 100) chance of that
    /// radius being drawn.
    fn new<I>(items: I) -> Self
    where
        I: IntoIterator<Item = (f64, u8)>,
    {
        let (choices, weights): (Vec<f64>, Vec<u8>) = items.into_iter().unzip();
        let dist = WeightedIndex::new(weights).unwrap();
        WeightedRadiusDist { choices, dist }
    }
}

/// A struct containing the output of one sphere packing simulation,
/// consisting of both volume fraction and surface area to volume ratio.
#[derive(Serialize)]
pub(crate) struct SimOutput {
    /// Packing efficiency fraction
    volume_fraction: f64,
    /// Surface area to volume ratio
    sa_to_vol: f64,
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
    // Compute boundaries, attempting to fit in at least 1000 spheres with a 50% packing volume, into
    // a cylinder with aspect ratio (height / radius) of 8.
    const TARGET_SPHERE_CT: f64 = 1000.;
    let sphere_volume = ((spheres.total_volume() / 100.) * TARGET_SPHERE_CT) as f32;
    let cyl_volume = (sphere_volume * 2.) as f32;
    const ASPECT_RATIO: f32 = 8.;
    let cyl_rad = (cyl_volume / (PI * ASPECT_RATIO)).cbrt();
    let cyl_height = cyl_rad * ASPECT_RATIO;
    let container = Cylinder::new(Point3::origin(), cyl_rad as f32, cyl_height as f32);
    let mut sizes = WeightedRadiusDist::new(spheres.iter().map(|s| (s.radius(), s.proportion())));
    let packed = PackedVolume::new(container, &mut sizes)?;
    Ok(SimOutput {
        volume_fraction: packed.volume_fraction() as f64,
        sa_to_vol: spheres.total_volume() / spheres.total_surface_area(),
    })
}
