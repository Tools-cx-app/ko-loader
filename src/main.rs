mod assets;

use std::{env, ffi::CString, path::Path, process::Command};

use anyhow::{Context, Result, anyhow, bail};
use regex_lite::Regex;
use rustix::system::init_module;

fn parse_kmi(version: &str) -> Result<String> {
    let re = Regex::new(r"(.* )?(\d+\.\d+)(\S+)?(android\d+)(.*)")?;
    let cap = re
        .captures(version)
        .ok_or_else(|| anyhow::anyhow!("Failed to get KMI from boot/modules"))?;
    let android_version = cap.get(4).map_or("", |m| m.as_str());
    let kernel_version = cap.get(2).map_or("", |m| m.as_str());
    Ok(format!("{android_version}-{kernel_version}"))
}

fn parse_kmi_from_uname() -> Result<String> {
    let uname = rustix::system::uname();
    let version = uname.release().to_string_lossy();
    parse_kmi(&version)
}

fn parse_kmi_from_modules() -> Result<String> {
    use std::io::BufRead;
    // find a *.ko in /vendor/lib/modules
    let modfile = std::fs::read_dir("/vendor/lib/modules")?
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "ko"))
        .map(|entry| entry.path())
        .ok_or_else(|| anyhow!("No kernel module found"))?;
    let output = Command::new("modinfo").arg(modfile).output()?;
    for line in output.stdout.lines().map_while(Result::ok) {
        if line.starts_with("vermagic") {
            return parse_kmi(&line);
        }
    }
    bail!("Parse KMI from modules failed")
}

fn get_current_kmi() -> Result<String> {
    parse_kmi_from_uname().or_else(|_| parse_kmi_from_modules())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if let Some(module) = args.get(1) {
        let module = Path::new(module);
        let module = module
            .canonicalize()
            .with_context(|| format!("resolve module path failed: {}", module.display()))?;
        let kmis = assets::list_supported_kmi();
        println!("ko: {kmis:?}");
        let kmi = get_current_kmi()?;

        if !kmis.contains(&kmi) {
            bail!("unsupported kmi!!");
        }

        let params = CString::new(format!("module_path={}", module.display()))?;
        init_module(
            &assets::get_asset_data(&format!("{kmi}-lkmloader.ko")).unwrap(),
            params.as_c_str(),
        )?;

        println!("Loaded kernel module: {}", module.display());
    } else {
        eprintln!("Usages: {} MODULE", args.get(0).unwrap());
        eprintln!("  Load the module named MODULE passing options if given.");

        std::process::exit(1);
    }

    Ok(())
}
