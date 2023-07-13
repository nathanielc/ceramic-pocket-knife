use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    let path = generate_to(
        Bash,
        &mut cmd,        // We need to specify what generator to use
        "cpk",           // We need to specify the bin name manually
        outdir,          // We need to specify where to write to
    )?;

    println!("cargo:warning=completion file is generated: {:?}", path);

    Ok(())
}
