//! Module used for parsing JSON into sphere packing parameters.
use std::f64::consts::PI;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
/// An enumeration of the different errors that can occur while parsing a string into a Spheres
/// struct.
pub(crate) enum ParsingError {
    #[error("failed to parse string")]
    FailedToParse(#[from] serde_json::Error),
    #[error("non-positive values for radius are not allowed")]
    NonPositive,
    #[error("invalid proportions: did not sum to 100")]
    InvalidProportions,
}

#[derive(Debug, Deserialize, PartialEq)]
/// A struct representing a sphere that has not been validated yet.
struct SpheresRaw(Vec<ParsedSphere>);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
/// A struct representing the properties of a single sphere type.
pub(crate) struct ParsedSphere {
    name: String,
    radius: f64,
    proportion: u8,
}

impl ParsedSphere {
    pub(crate) fn radius(&self) -> f64 {
        self.radius
    }

    pub(crate) fn proportion(&self) -> u8 {
        self.proportion
    }
}

#[derive(Debug, Serialize, PartialEq)]
/// A struct representing multiple spheres to attempt to pack, after validation.
pub(crate) struct Spheres(Vec<ParsedSphere>);

impl FromStr for SpheresRaw {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

/// Validate `raw`, providing the corresponding validated Spheres struct or an error if `raw` is
/// invalid.
///
/// `raw` is invalid if the sum of its proportions is not exactly 100, or if any radii are less than
/// or equal to 0 (proportions must be at least 0).
fn validate(raw: SpheresRaw) -> Result<Spheres, ParsingError> {
    if raw.0.iter().all(|s| s.radius > 0.0) {
        if raw.0.iter().map(|s| s.proportion as u32).sum::<u32>() == 100 {
            Ok(Spheres(raw.0))
        } else {
            Err(ParsingError::InvalidProportions)
        }
    } else {
        Err(ParsingError::NonPositive)
    }
}

impl FromStr for Spheres {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate(s.parse::<SpheresRaw>()?)
    }
}

impl Spheres {
    /// Provides an iterator over the spheres contained by this struct.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &ParsedSphere> {
        self.0.iter()
    }

    pub(crate) fn avg_volume(&self) -> f64 {
        self.0
            .iter()
            .map(|s| 4.0 / 3.0 * PI * s.radius.powi(3) * (s.proportion as f64 / 100.))
            .sum()
    }

    pub(crate) fn avg_surface_area(&self) -> f64 {
        self.0
            .iter()
            .map(|s| 4.0 * PI * s.radius.powi(2) * (s.proportion as f64 / 100.))
            .sum()
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::{validate, ParsedSphere, ParsingError, Spheres, SpheresRaw};

    static VALID: &str = r#"
[
  {
    "name": "5_micron_Al",
    "radius": 5.0,
    "proportion": 66
  },
  {
    "name": "400_AP",
    "radius": 400,
    "proportion": 34
  }
]"#;

    fn valid_spheres_raw() -> SpheresRaw {
        SpheresRaw(vec![
            ParsedSphere {
                name: String::from("5_micron_Al"),
                radius: 5.0,
                proportion: 66,
            },
            ParsedSphere {
                name: String::from("400_AP"),
                radius: 400.0,
                proportion: 34,
            },
        ])
    }

    fn valid_spheres() -> Spheres {
        Spheres(vec![
            ParsedSphere {
                name: String::from("5_micron_Al"),
                radius: 5.0,
                proportion: 66,
            },
            ParsedSphere {
                name: String::from("400_AP"),
                radius: 400.0,
                proportion: 34,
            },
        ])
    }

    static INVALID: &str = r#"
[
  {
    "name": "5_micron_Al",
    "radius": 5.0,
    "proportion": 66
  },
  {
    "name": "400_AP",
    "radius": 400,
    "proportion": 32
  }
]"#;

    fn invalid_spheres() -> SpheresRaw {
        SpheresRaw(vec![
            ParsedSphere {
                name: String::from("5_micron_Al"),
                radius: 5.0,
                proportion: 66,
            },
            ParsedSphere {
                name: String::from("400_AP"),
                radius: 400.0,
                proportion: 32,
            },
        ])
    }

    static MALFORMED: &str = r#"
    [
  {
    "name": "5_micron_Al",
    "radius": 5.0,
  },
  {
    "name": "400_AP",
    "radius": 400,
    "proportion": 32
  }
]"#;

    static NEG_RADIUS: &str = r#"
    [
  {
    "name": "5_micron_Al",
    "radius": -5.0,
    "proportion": 100
  }
    ]
    "#;

    #[test]
    fn parse_well_formed() {
        assert_eq!(valid_spheres_raw(), VALID.parse().unwrap())
    }

    #[test]
    fn parse_invalid_well_formed() {
        assert_eq!(invalid_spheres(), INVALID.parse().unwrap())
    }

    #[test]
    fn validate_well_formed() {
        assert_eq!(valid_spheres(), validate(valid_spheres_raw()).unwrap())
    }

    #[test]
    fn parse_malformed() {
        assert!(matches!(
            MALFORMED.parse::<SpheresRaw>(),
            Err(ParsingError::FailedToParse(_))
        ))
    }

    #[test]
    fn validate_invalid() {
        assert!(matches!(
            validate(invalid_spheres()),
            Err(ParsingError::InvalidProportions)
        ))
    }

    #[test]
    fn try_into_valid() {
        assert_eq!(valid_spheres(), VALID.parse().unwrap());
    }

    #[test]
    fn try_into_invalid() {
        assert!(matches!(
            INVALID.parse::<Spheres>(),
            Err(ParsingError::InvalidProportions)
        ))
    }

    #[test]
    fn try_into_malformed() {
        assert!(matches!(
            MALFORMED.parse::<Spheres>(),
            Err(ParsingError::FailedToParse(_))
        ))
    }

    #[test]
    fn negative_radius() {
        let neg_rad = NEG_RADIUS.parse();
        assert!(neg_rad.is_ok());
        assert!(matches!(
            validate(neg_rad.unwrap()),
            Err(ParsingError::NonPositive)
        ));
        assert!(matches!(
            NEG_RADIUS.parse::<Spheres>(),
            Err(ParsingError::NonPositive)
        ))
    }
}
