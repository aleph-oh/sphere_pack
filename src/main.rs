use clap::clap_app;

mod packing;
mod parsing;

use std::fs;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(pack =>
        (name: "pack")
        (version: "0.1")
        (about: "Attempts to pack spheres into a cube and reports result")
        (@arg input: +required "Sets the input JSON file to use")
        (@arg output: +required "Sets the filename of the output JSON file")
    )
    .get_matches();
    let input = matches.value_of("input").unwrap();
    let json =
        fs::read_to_string(input)?;
    let spheres = json.parse()?;
    let sim_result = packing::pack(&spheres)?;
    let output = matches.value_of("output").unwrap();
    fs::write(output, serde_json::to_string(&sim_result)?)?;
    Ok(())
}
