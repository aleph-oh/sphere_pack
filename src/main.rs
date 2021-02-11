#[macro_use]
extern crate clap;
extern crate spherical_cow;

mod packing;
mod parsing;

use std::fs;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::clap_app!(pack =>
        (version: "0.1")
        (about: "Attempts to pack spheres into a cylinder and reports result")
        (@arg input: +required "Sets the input JSON file to use")
        (@arg output: +required "Sets the filename of the output JSON file")
    )
    .get_matches();
    let input = matches.value_of("input").unwrap();
    let json =
        fs::read_to_string(input).expect(format!("Failed to open input file {}", input).as_str());
    let spheres = json.parse()?;
    let sim_result = packing::pack(&spheres)?;
    let output = matches.value_of("output").unwrap();
    fs::write(output, serde_json::to_string(&sim_result)?)
        .expect(format!("Failed to write to output file {}", output).as_str());
    Ok(())
}
