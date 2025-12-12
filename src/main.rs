use csv::Writer;
use serde::{Deserialize, Serialize};
use tap::Pipe;
use std::fs::File;
use std::process::Command;
use std::io::{self, BufRead, BufReader};



// pkg license information
#[derive(Debug, Default, Serialize)]
struct Pkg {
    name: String,
    #[serde(flatten)]
    license: License,
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct License {
    deprecated: bool,
    free: bool,
    #[serde(rename = "fullName")]
    full_name: String,
    redistributable: bool,
    #[serde(rename = "shortName")]
    short_name: String,
    #[serde(rename = "spdxId")]
    spdx_id: String,
    url: String,
}




fn main() -> io::Result<()> {
    let mut wtr = Writer::from_path("./data/pkgs.csv")?;
    let pkgslist: File = File::open("./data/pkgs.txt")?;

    // Header
    wtr.write_record(&["name", "fullName", "shortName", "spdxId", "url"])?;

    // Stream lines, capture license info, and serialize each pkg
    BufReader::new(pkgslist)
        .lines()
        .filter_map(Result::ok) // drop bad lines
        .map(|pkg_name| capture_license_info(&pkg_name)) // run function
        .filter_map(Result::ok) // drop errors
        .try_for_each(|pkg| write_csv_record(pkg, &mut wtr))?; // consume iterator, propagate errors

    wtr.flush()?;
    Ok(())
}


fn run_nix_eval(pkgs_name: &str) -> Result<String, io::Error> {
    let output = Command::new("nix")
        .arg("eval")
        .arg("--json")
        .arg(format!("nixpkgs#{}.meta.license", pkgs_name))
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_owned())
}

fn capture_license_info(pkgs_name: &str) -> Result<Pkg, io::Error> {
    let license: License = run_nix_eval(pkgs_name)?
        .pipe(|l| serde_json::from_str(&l)) // returns Result<License, serde_json::Error>
        .map_err(io::Error::other)?; // map serde error into io::Error

    Ok(Pkg {
        name: pkgs_name.to_owned(),
        license,
    })
}

fn write_csv_record(pkg: Pkg, wtr: &mut Writer<File>) -> Result<(), io::Error> {
    wtr.write_record(&[
        pkg.name,
        pkg.license.full_name,
        pkg.license.short_name,
        pkg.license.spdx_id,
        pkg.license.url,
    ])?;

    Ok(())
}
