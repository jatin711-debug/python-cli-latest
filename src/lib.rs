use clap::Subcommand;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageRegistry {
    pub packages: HashMap<String, Package>,
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Install {
        packages: Vec<String>,
        #[arg(short = 'p', long = "parallel", help = "Install packages in parallel")]
        parallel: bool,
    },
    Delete {
        name: String,
    },
    Update {
        name: String,
        version: String,
    },
    List,
}

fn get_python_executable() -> String {
    if cfg!(test) {
        // Return mock path during tests
        "mock_python".to_string()
    } else {
        let commands = ["python3", "python"];

        for cmd in &commands {
            let output = Command::new(cmd)
                .arg("-c")
                .arg("import sys; print(sys.executable)")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    return String::from_utf8(output.stdout)
                        .unwrap_or_else(|_| panic!("Invalid UTF-8 in Python path"))
                        .trim()
                        .to_string();
                }
            }
        }
        panic!("Failed to find Python executable");
    }
}

fn get_installed_version(python: &str, name: &str) -> String {
    let output = Command::new(python)
        .arg("-m")
        .arg("pip")
        .arg("show")
        .arg(name)
        .output()
        .unwrap_or_else(|_| panic!("Failed to check version for {}", name));

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|l| l.starts_with("Version: "))
            .map(|l| l.replace("Version: ", "").trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "unknown".to_string()
    }
}

pub fn load_packages() -> PackageRegistry {
    let path = PathBuf::from("packages.json");
    if path.exists() {
        let file =
            File::open(&path).unwrap_or_else(|_| panic!("Failed to open {}", path.display()));
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| PackageRegistry {
            packages: HashMap::new(),
        })
    } else {
        PackageRegistry {
            packages: HashMap::new(),
        }
    }
}

pub fn save_packages(registry: &PackageRegistry) {
    let path = PathBuf::from("packages.json");
    let file =
        File::create(&path).unwrap_or_else(|_| panic!("Failed to create {}", path.display()));
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, registry).expect("Failed to write packages");
}

pub fn install_packages(packages: &[String], registry: &mut PackageRegistry) {
    let python = get_python_executable();

    let package_specs: Vec<String> = packages
        .iter()
        .map(|pkg| {
            let (name, version) = parse_package_spec(pkg);
            version.map_or(name.clone(), |v| format!("{}=={}", name, v))
        })
        .collect();

    let status = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .args(&package_specs)
        .status()
        .expect("Failed to execute pip install");

    if status.success() {
        for spec in packages {
            let (name, version_option) = parse_package_spec(spec);
            let version = version_option.unwrap_or_else(|| get_installed_version(&python, &name));

            registry.packages.insert(
                name.clone(),
                Package {
                    name: name.clone(),
                    version: version.clone(),
                },
            );
            println!("Installed {} {}", name, version);
        }
    } else {
        eprintln!("Failed to install packages");
    }
}

pub fn install_packages_parallel(packages: &[String], registry: &mut PackageRegistry) {
    if packages.is_empty() {
        return;
    }

    let python = get_python_executable();

    // Create progress bar
    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Thread-safe registry wrapper
    let registry_mutex = Arc::new(Mutex::new(registry));
    let results: Vec<Result<(String, String, String), String>> = packages
        .par_iter()
        .map(|pkg| {
            let (name, version) = parse_package_spec(pkg);
            let package_spec = version
                .as_ref()
                .map_or(name.clone(), |v| format!("{}=={}", name, v));

            pb.set_message(format!("Installing {}", name));

            // Install individual package
            let status = Command::new(&python)
                .arg("-m")
                .arg("pip")
                .arg("install")
                .arg(&package_spec)
                .output();

            match status {
                Ok(output) => {
                    if output.status.success() {
                        let installed_version =
                            version.unwrap_or_else(|| get_installed_version(&python, &name));
                        pb.inc(1);
                        Ok((name.clone(), installed_version, package_spec))
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        pb.inc(1);
                        Err(format!("Failed to install {}: {}", name, error))
                    }
                }
                Err(e) => {
                    pb.inc(1);
                    Err(format!("Failed to execute pip for {}: {}", name, e))
                }
            }
        })
        .collect();

    pb.finish_with_message("Installation complete");

    // Process results and update registry
    let mut success_count = 0;
    let mut failure_count = 0;

    for result in results {
        match result {
            Ok((name, version, _)) => {
                let mut reg = registry_mutex.lock().unwrap();
                reg.packages.insert(
                    name.clone(),
                    Package {
                        name: name.clone(),
                        version: version.clone(),
                    },
                );
                println!("✓ Successfully installed {} {}", name, version);
                success_count += 1;
            }
            Err(error) => {
                eprintln!("✗ {}", error);
                failure_count += 1;
            }
        }
    }

    println!(
        "\nInstallation summary: {} succeeded, {} failed",
        success_count, failure_count
    );
}

pub fn delete_package(name: &str, registry: &mut PackageRegistry) {
    let python = get_python_executable();

    let status = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("uninstall")
        .arg(name)
        .arg("-y")
        .status()
        .expect("Failed to execute pip uninstall");

    if status.success() {
        registry.packages.remove(name);
        println!("Removed package {}", name);
    } else {
        eprintln!("Failed to uninstall package {}", name);
    }
}

pub fn update_package(name: &str, version: &str, registry: &mut PackageRegistry) {
    let python = get_python_executable();

    let status = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("--upgrade")
        .arg(format!("{}=={}", name, version))
        .status()
        .expect("Failed to execute pip install");

    if status.success() {
        let installed_version = get_installed_version(&python, name);
        registry.packages.insert(
            name.to_string(),
            Package {
                name: name.to_string(),
                version: installed_version.clone(),
            },
        );
        println!("Updated {} to version {}", name, installed_version);
    } else {
        eprintln!("Failed to update package {}", name);
    }
}

pub fn list_packages(registry: &PackageRegistry) {
    if registry.packages.is_empty() {
        println!("No packages installed");
        return;
    }

    println!("Installed packages:");
    for (name, pkg) in &registry.packages {
        println!("- {} @ {}", name, pkg.version);
    }
}

pub fn install_from_requirements(path: &str, registry: &mut PackageRegistry) {
    install_from_requirements_impl(path, registry, false);
}

pub fn install_from_requirements_parallel(path: &str, registry: &mut PackageRegistry) {
    install_from_requirements_impl(path, registry, true);
}

fn install_from_requirements_impl(path: &str, registry: &mut PackageRegistry, parallel: bool) {
    let file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
    let reader = BufReader::new(file);

    let mut packages = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Error reading line");
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        packages.push(line.to_string());
    }

    if parallel {
        install_packages_parallel(&packages, registry);
    } else {
        let python = get_python_executable();

        let status = Command::new(&python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-r")
            .arg(path)
            .status()
            .expect("Failed to execute pip install");

        if status.success() {
            for spec in packages {
                let (name, version_option) = parse_package_spec(&spec);
                let version =
                    version_option.unwrap_or_else(|| get_installed_version(&python, &name));

                registry.packages.insert(
                    name.clone(),
                    Package {
                        name: name.clone(),
                        version: version.clone(),
                    },
                );
                println!("Installed {} {}", name, version);
            }
        } else {
            eprintln!("Failed to install packages from {}", path);
        }
    }
}

fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = spec.splitn(2, "==").collect();
    match parts.as_slice() {
        [name, version] => (name.to_string(), Some(version.to_string())),
        [name] => (name.to_string(), None),
        _ => panic!("Invalid package specification: {}", spec),
    }
}
