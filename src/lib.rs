//! Python Package Manager CLI Library
//!
//! This library provides a command-line interface for managing Python packages
//! with support for parallel installation, requirements file processing, and
//! package registry management.

use clap::Subcommand;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{fmt, result};

/// Custom error type for package management operations
#[derive(Debug)]
pub enum PackageError {
    /// IO operation failed
    IoError(std::io::Error),
    /// Python executable not found
    PythonNotFound,
    /// Package installation failed
    InstallationFailed(String),
    /// Package uninstallation failed
    UninstallationFailed(String),
    /// Invalid package specification
    InvalidPackageSpec(String),
    /// JSON serialization/deserialization failed
    JsonError(serde_json::Error),
    /// Package not found in registry
    PackageNotFound(String),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageError::IoError(e) => write!(f, "IO error: {}", e),
            PackageError::PythonNotFound => write!(f, "Python executable not found"),
            PackageError::InstallationFailed(msg) => write!(f, "Installation failed: {}", msg),
            PackageError::UninstallationFailed(msg) => write!(f, "Uninstallation failed: {}", msg),
            PackageError::InvalidPackageSpec(spec) => write!(f, "Invalid package spec: {}", spec),
            PackageError::JsonError(e) => write!(f, "JSON error: {}", e),
            PackageError::PackageNotFound(name) => write!(f, "Package not found: {}", name),
        }
    }
}

impl std::error::Error for PackageError {}

impl From<std::io::Error> for PackageError {
    fn from(error: std::io::Error) -> Self {
        PackageError::IoError(error)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(error: serde_json::Error) -> Self {
        PackageError::JsonError(error)
    }
}

/// Custom Result type for package operations
pub type Result<T> = result::Result<T, PackageError>;

/// Represents a Python package with its name and version
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Package {
    /// Package name as it appears in PyPI
    pub name: String,
    /// Installed version of the package
    pub version: String,
}

impl Package {
    /// Creates a new Package instance
    ///
    /// # Arguments
    /// * `name` - The package name
    /// * `version` - The package version
    ///
    /// # Returns
    /// A new Package instance
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

/// Registry for tracking installed packages
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PackageRegistry {
    /// Map of package names to Package instances
    pub packages: HashMap<String, Package>,
}

impl PackageRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Adds a package to the registry
    ///
    /// # Arguments
    /// * `package` - The package to add
    pub fn add_package(&mut self, package: Package) {
        self.packages.insert(package.name.clone(), package);
    }

    /// Removes a package from the registry
    ///
    /// # Arguments
    /// * `name` - The name of the package to remove
    ///
    /// # Returns
    /// The removed package if it existed
    pub fn remove_package(&mut self, name: &str) -> Option<Package> {
        self.packages.remove(name)
    }

    /// Gets a package from the registry
    ///
    /// # Arguments
    /// * `name` - The name of the package to retrieve
    ///
    /// # Returns
    /// Reference to the package if it exists
    pub fn get_package(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }

    /// Checks if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }
}

/// Command line interface structure
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Install Python packages
    Install {
        /// List of packages to install (can include version specs like "package==1.0.0")
        packages: Vec<String>,
        /// Install packages in parallel for faster execution
        #[arg(short = 'p', long = "parallel", help = "Install packages in parallel")]
        parallel: bool,
    },
    /// Delete a Python package
    Delete {
        /// Name of the package to delete
        name: String,
    },
    /// Update a Python package to a specific version
    Update {
        /// Name of the package to update
        name: String,
        /// Target version for the update
        version: String,
    },
    /// List all installed packages
    List,
}

/// Trait defining package management operations
pub trait PackageManager {
    /// Installs packages sequentially
    fn install_packages(&self, packages: &[String], registry: &mut PackageRegistry) -> Result<()>;

    /// Installs packages in parallel
    fn install_packages_parallel(
        &self,
        packages: &[String],
        registry: &mut PackageRegistry,
    ) -> Result<()>;

    /// Deletes a single package
    fn delete_package(&self, name: &str, registry: &mut PackageRegistry) -> Result<()>;

    /// Updates a package to a specific version
    fn update_package(
        &self,
        name: &str,
        version: &str,
        registry: &mut PackageRegistry,
    ) -> Result<()>;

    /// Lists all packages in the registry
    fn list_packages(&self, registry: &PackageRegistry);

    /// Installs packages from a requirements file
    fn install_from_requirements(&self, path: &str, registry: &mut PackageRegistry) -> Result<()>;

    /// Installs packages from a requirements file in parallel
    fn install_from_requirements_parallel(
        &self,
        path: &str,
        registry: &mut PackageRegistry,
    ) -> Result<()>;
}

impl PackageManager for Cli {
    fn install_packages(&self, packages: &[String], registry: &mut PackageRegistry) -> Result<()> {
        install_packages(packages, registry)
    }

    fn install_packages_parallel(
        &self,
        packages: &[String],
        registry: &mut PackageRegistry,
    ) -> Result<()> {
        install_packages_parallel(packages, registry)
    }

    fn delete_package(&self, name: &str, registry: &mut PackageRegistry) -> Result<()> {
        delete_package(name, registry)
    }

    fn update_package(
        &self,
        name: &str,
        version: &str,
        registry: &mut PackageRegistry,
    ) -> Result<()> {
        update_package(name, version, registry)
    }

    fn list_packages(&self, registry: &PackageRegistry) {
        list_packages(registry)
    }

    fn install_from_requirements(&self, path: &str, registry: &mut PackageRegistry) -> Result<()> {
        install_from_requirements(path, registry)
    }

    fn install_from_requirements_parallel(
        &self,
        path: &str,
        registry: &mut PackageRegistry,
    ) -> Result<()> {
        install_from_requirements_parallel(path, registry)
    }
}

/// Locates the Python executable on the system
///
/// This function attempts to find a valid Python executable by trying
/// common command names in order of preference. It validates that the
/// found executable is actually working by running a simple command.
///
/// # Returns
/// * `Result<String>` - Path to the Python executable or error if not found
///
/// # Examples
/// ```
/// let python_path = get_python_executable().unwrap();
/// println!("Using Python: {}", python_path);
/// ```
fn get_python_executable() -> Result<String> {
    if cfg!(test) {
        return Ok("mock_python".to_string());
    }

    let candidates = ["python3", "python", "py"];

    for cmd in &candidates {
        if let Ok(output) = Command::new(cmd)
            .arg("-c")
            .arg("import sys; print(sys.executable)")
            .output()
        {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    return Ok(path.trim().to_string());
                }
            }
        }
    }

    Err(PackageError::PythonNotFound)
}

/// Retrieves the installed version of a specific package
///
/// Uses `pip show` command to query the installed version of a package.
/// Returns "unknown" if the package is not installed or version cannot be determined.
///
/// # Arguments
/// * `python` - Path to the Python executable
/// * `name` - Name of the package to check
///
/// # Returns
/// * `Result<String>` - Version string or "unknown" if not found
fn get_installed_version(python: &str, name: &str) -> Result<String> {
    let output = Command::new(python)
        .arg("-m")
        .arg("pip")
        .arg("show")
        .arg(name)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = stdout
            .lines()
            .find(|line| line.starts_with("Version: "))
            .and_then(|line| line.strip_prefix("Version: "))
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        Ok(version)
    } else {
        Ok("unknown".to_string())
    }
}

/// Loads the package registry from the JSON file
///
/// Attempts to load the package registry from `packages.json` in the current directory.
/// If the file doesn't exist or is corrupted, returns an empty registry.
///
/// # Returns
/// * `Result<PackageRegistry>` - Loaded registry or empty registry on error
pub fn load_packages() -> Result<PackageRegistry> {
    let path = PathBuf::from("packages.json");

    if !path.exists() {
        return Ok(PackageRegistry::new());
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    match serde_json::from_reader(reader) {
        Ok(registry) => Ok(registry),
        Err(_) => {
            eprintln!("Warning: Corrupted packages.json file, starting with empty registry");
            Ok(PackageRegistry::new())
        }
    }
}

/// Saves the package registry to the JSON file
///
/// Serializes the current package registry to `packages.json` in the current directory.
/// Uses pretty printing for better readability.
///
/// # Arguments
/// * `registry` - The registry to save
///
/// # Returns
/// * `Result<()>` - Success or IO error
pub fn save_packages(registry: &PackageRegistry) -> Result<()> {
    let path = PathBuf::from("packages.json");
    let file = File::create(&path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, registry)?;
    Ok(())
}

/// Installs packages sequentially using pip
///
/// Installs the specified packages one by one using a single pip command.
/// Updates the registry with the installed packages and their versions.
///
/// # Arguments
/// * `packages` - Slice of package specifications to install
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or installation error
pub fn install_packages(packages: &[String], registry: &mut PackageRegistry) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let python = get_python_executable()?;
    let package_specs = prepare_package_specs(packages)?;

    println!("Installing packages: {}", package_specs.join(", "));

    let output = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .args(&package_specs)
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::InstallationFailed(error_msg.to_string()));
    }

    // Update registry with installed packages
    for spec in packages {
        let (name, version_option) = parse_package_spec(spec)?;
        let version = match version_option {
            Some(v) => v,
            None => get_installed_version(&python, &name)?,
        };

        let package = Package::new(name.clone(), version.clone());
        registry.add_package(package);
        println!("✓ Successfully installed {} {}", name, version);
    }

    Ok(())
}

/// Installs packages in parallel using rayon
///
/// Installs each package in a separate thread for faster execution.
/// Provides a progress bar to show installation progress.
///
/// # Arguments
/// * `packages` - Slice of package specifications to install
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or installation error
pub fn install_packages_parallel(
    packages: &[String],
    registry: &mut PackageRegistry,
) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let python = get_python_executable()?;

    // Create and configure progress bar
    let pb = create_progress_bar(packages.len());

    // Thread-safe registry wrapper
    let registry_mutex = Arc::new(Mutex::new(registry));

    // Install packages in parallel
    let results: Vec<Result<(String, String)>> = packages
        .par_iter()
        .map(|pkg| {
            let result = install_single_package(&python, pkg, &pb);
            pb.inc(1);
            result
        })
        .collect();

    pb.finish_with_message("Installation complete");

    // Process results and update registry
    process_installation_results(results, registry_mutex)?;

    Ok(())
}

/// Deletes a package using pip uninstall
///
/// Removes the specified package from the system and updates the registry.
///
/// # Arguments
/// * `name` - Name of the package to delete
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or deletion error
pub fn delete_package(name: &str, registry: &mut PackageRegistry) -> Result<()> {
    if name.trim().is_empty() {
        return Err(PackageError::InvalidPackageSpec(
            "Package name cannot be empty".to_string(),
        ));
    }

    let python = get_python_executable()?;

    let output = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("uninstall")
        .arg(name)
        .arg("-y")
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::UninstallationFailed(error_msg.to_string()));
    }

    registry.remove_package(name);
    println!("✓ Successfully removed package {}", name);
    Ok(())
}

/// Updates a package to a specific version
///
/// Uses pip install --upgrade to update the package to the specified version.
///
/// # Arguments
/// * `name` - Name of the package to update
/// * `version` - Target version for the update
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or update error
pub fn update_package(name: &str, version: &str, registry: &mut PackageRegistry) -> Result<()> {
    if name.trim().is_empty() || version.trim().is_empty() {
        return Err(PackageError::InvalidPackageSpec(
            "Package name and version cannot be empty".to_string(),
        ));
    }

    let python = get_python_executable()?;
    let package_spec = format!("{}=={}", name, version);

    let output = Command::new(&python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("--upgrade")
        .arg(&package_spec)
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::InstallationFailed(error_msg.to_string()));
    }

    let installed_version = get_installed_version(&python, name)?;
    let package = Package::new(name.to_string(), installed_version.clone());
    registry.add_package(package);

    println!(
        "✓ Successfully updated {} to version {}",
        name, installed_version
    );
    Ok(())
}

/// Lists all packages in the registry
///
/// Displays all installed packages with their versions in a formatted list.
///
/// # Arguments
/// * `registry` - Reference to the package registry
pub fn list_packages(registry: &PackageRegistry) {
    if registry.is_empty() {
        println!("No packages installed");
        return;
    }

    println!("Installed packages ({} total):", registry.packages.len());
    let mut packages: Vec<_> = registry.packages.values().collect();
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    for package in packages {
        println!("  {} @ {}", package.name, package.version);
    }
}

/// Installs packages from a requirements file
///
/// Reads a requirements.txt file and installs all specified packages.
///
/// # Arguments
/// * `path` - Path to the requirements file
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or installation error
pub fn install_from_requirements(path: &str, registry: &mut PackageRegistry) -> Result<()> {
    install_from_requirements_impl(path, registry, false)
}

/// Installs packages from a requirements file in parallel
///
/// Reads a requirements.txt file and installs all specified packages in parallel.
///
/// # Arguments
/// * `path` - Path to the requirements file
/// * `registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or installation error
pub fn install_from_requirements_parallel(
    path: &str,
    registry: &mut PackageRegistry,
) -> Result<()> {
    install_from_requirements_impl(path, registry, true)
}

// Helper functions

/// Creates a configured progress bar for package installation
fn create_progress_bar(len: usize) -> ProgressBar {
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

/// Installs a single package and returns the result
fn install_single_package(python: &str, pkg: &str, pb: &ProgressBar) -> Result<(String, String)> {
    let (name, version) = parse_package_spec(pkg)?;
    let package_spec = version
        .as_ref()
        .map_or(name.clone(), |v| format!("{}=={}", name, v));

    pb.set_message(format!("Installing {}", name));

    let output = Command::new(python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg(&package_spec)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(PackageError::InstallationFailed(format!(
            "Failed to install {}: {}",
            name, error
        )));
    }

    let installed_version = version.unwrap_or_else(|| {
        get_installed_version(python, &name).unwrap_or_else(|_| "unknown".to_string())
    });

    Ok((name, installed_version))
}

/// Processes installation results and updates the registry
fn process_installation_results(
    results: Vec<Result<(String, String)>>,
    registry_mutex: Arc<Mutex<&mut PackageRegistry>>,
) -> Result<()> {
    let mut success_count = 0;
    let mut failure_count = 0;

    for result in results {
        match result {
            Ok((name, version)) => {
                let mut reg = registry_mutex.lock().unwrap();
                let package = Package::new(name.clone(), version.clone());
                reg.add_package(package);
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

    if failure_count > 0 {
        Err(PackageError::InstallationFailed(format!(
            "{} packages failed to install",
            failure_count
        )))
    } else {
        Ok(())
    }
}

/// Prepares package specifications for pip installation
fn prepare_package_specs(packages: &[String]) -> Result<Vec<String>> {
    packages
        .iter()
        .map(|pkg| {
            let (name, version) = parse_package_spec(pkg)?;
            Ok(version.map_or(name.clone(), |v| format!("{}=={}", name, v)))
        })
        .collect()
}

/// Implementation for installing from requirements files
fn install_from_requirements_impl(
    path: &str,
    registry: &mut PackageRegistry,
    parallel: bool,
) -> Result<()> {
    if !Path::new(path).exists() {
        return Err(PackageError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Requirements file not found: {}", path),
        )));
    }

    let packages = parse_requirements_file(path)?;

    if packages.is_empty() {
        println!("No packages found in requirements file");
        return Ok(());
    }

    println!("Installing {} packages from {}", packages.len(), path);

    if parallel {
        install_packages_parallel(&packages, registry)
    } else {
        install_packages(&packages, registry)
    }
}

/// Parses a requirements file and returns package specifications
fn parse_requirements_file(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut packages = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Basic validation of package specification
        if line.contains(' ') && !line.contains("==") {
            eprintln!("Warning: Skipping potentially invalid line: {}", line);
            continue;
        }

        packages.push(line.to_string());
    }

    Ok(packages)
}

/// Parses a package specification into name and optional version
///
/// Supports formats like "package" or "package==1.0.0"
///
/// # Arguments
/// * `spec` - Package specification string
///
/// # Returns
/// * `Result<(String, Option<String>)>` - Package name and optional version
fn parse_package_spec(spec: &str) -> Result<(String, Option<String>)> {
    let spec = spec.trim();

    if spec.is_empty() {
        return Err(PackageError::InvalidPackageSpec(
            "Empty package specification".to_string(),
        ));
    }

    let parts: Vec<&str> = spec.splitn(2, "==").collect();
    match parts.as_slice() {
        [name, version] => {
            let name = name.trim();
            let version = version.trim();

            if name.is_empty() || version.is_empty() {
                return Err(PackageError::InvalidPackageSpec(format!(
                    "Invalid package specification: {}",
                    spec
                )));
            }

            Ok((name.to_string(), Some(version.to_string())))
        }
        [name] => {
            let name = name.trim();
            if name.is_empty() {
                return Err(PackageError::InvalidPackageSpec(
                    "Empty package name".to_string(),
                ));
            }
            Ok((name.to_string(), None))
        }
        _ => Err(PackageError::InvalidPackageSpec(format!(
            "Invalid package specification: {}",
            spec
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec_with_version() {
        let result = parse_package_spec("numpy==1.21.0").unwrap();
        assert_eq!(result, ("numpy".to_string(), Some("1.21.0".to_string())));
    }

    #[test]
    fn test_parse_package_spec_without_version() {
        let result = parse_package_spec("requests").unwrap();
        assert_eq!(result, ("requests".to_string(), None));
    }

    #[test]
    fn test_parse_package_spec_invalid() {
        assert!(parse_package_spec("").is_err());
        assert!(parse_package_spec("==1.0.0").is_err());
        assert!(parse_package_spec("package==").is_err());
    }

    #[test]
    fn test_package_registry_operations() {
        let mut registry = PackageRegistry::new();
        let package = Package::new("test".to_string(), "1.0.0".to_string());

        registry.add_package(package.clone());
        assert_eq!(registry.get_package("test"), Some(&package));

        let removed = registry.remove_package("test");
        assert_eq!(removed, Some(package));
        assert!(registry.is_empty());
    }
}
