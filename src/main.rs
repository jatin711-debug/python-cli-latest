use clap::Parser;
use python_package_manager::{
    delete_package, install_from_requirements, install_from_requirements_parallel,
    install_packages, install_packages_parallel, list_packages, load_packages, save_packages,
    update_package, Cli, Commands, PackageError,
};
use std::process;

/// Main entry point for the Python Package Manager CLI
///
/// Handles command parsing, package registry management, and error handling.
/// Provides appropriate exit codes for different error conditions.
fn main() {
    let args = Cli::parse();

    // Load package registry with error handling
    let mut package_registry = match load_packages() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("Error loading package registry: {}", e);
            process::exit(1);
        }
    };

    // Execute the requested command
    let result = match args.command {
        Commands::Install { packages, parallel } => {
            handle_install_command(packages, parallel, &mut package_registry)
        }
        Commands::Delete { name } => handle_delete_command(&name, &mut package_registry),
        Commands::Update { name, version } => {
            handle_update_command(&name, &version, &mut package_registry)
        }
        Commands::List => handle_list_command(&package_registry),
    };

    // Handle command execution results
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(get_exit_code(&e));
    }

    // Save package registry with error handling
    if let Err(e) = save_packages(&package_registry) {
        eprintln!("Warning: Failed to save package registry: {}", e);
        process::exit(2);
    }
}

/// Handles the install command with support for requirements files
///
/// # Arguments
/// * `packages` - List of package specifications or requirements file
/// * `parallel` - Whether to install packages in parallel
/// * `package_registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or error from installation
fn handle_install_command(
    packages: Vec<String>,
    parallel: bool,
    package_registry: &mut python_package_manager::PackageRegistry,
) -> Result<(), PackageError> {
    if packages.is_empty() {
        eprintln!("Error: No packages specified for installation");
        return Err(PackageError::InvalidPackageSpec(
            "No packages specified".to_string(),
        ));
    }

    // Check if this is a requirements file installation
    if packages.len() == 1 && packages[0].starts_with("-r=") {
        let requirements_path = &packages[0][3..];
        if requirements_path.is_empty() {
            return Err(PackageError::InvalidPackageSpec(
                "Empty requirements file path".to_string(),
            ));
        }

        println!("Installing from requirements file: {}", requirements_path);
        if parallel {
            install_from_requirements_parallel(requirements_path, package_registry)
        } else {
            install_from_requirements(requirements_path, package_registry)
        }
    } else {
        // Install individual packages
        println!("Installing {} package(s)...", packages.len());
        if parallel {
            install_packages_parallel(&packages, package_registry)
        } else {
            install_packages(&packages, package_registry)
        }
    }
}

/// Handles the delete command
///
/// # Arguments
/// * `name` - Name of the package to delete
/// * `package_registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or error from deletion
fn handle_delete_command(
    name: &str,
    package_registry: &mut python_package_manager::PackageRegistry,
) -> Result<(), PackageError> {
    if name.trim().is_empty() {
        return Err(PackageError::InvalidPackageSpec(
            "Package name cannot be empty".to_string(),
        ));
    }

    println!("Deleting package: {}", name);
    delete_package(name, package_registry)
}

/// Handles the update command
///
/// # Arguments
/// * `name` - Name of the package to update
/// * `version` - Target version for the update
/// * `package_registry` - Mutable reference to the package registry
///
/// # Returns
/// * `Result<()>` - Success or error from update
fn handle_update_command(
    name: &str,
    version: &str,
    package_registry: &mut python_package_manager::PackageRegistry,
) -> Result<(), PackageError> {
    if name.trim().is_empty() || version.trim().is_empty() {
        return Err(PackageError::InvalidPackageSpec(
            "Package name and version cannot be empty".to_string(),
        ));
    }

    println!("Updating package {} to version {}", name, version);
    update_package(name, version, package_registry)
}

/// Handles the list command
///
/// # Arguments
/// * `package_registry` - Reference to the package registry
///
/// # Returns
/// * `Result<()>` - Always succeeds for list command
fn handle_list_command(
    package_registry: &python_package_manager::PackageRegistry,
) -> Result<(), PackageError> {
    list_packages(package_registry);
    Ok(())
}

/// Maps package errors to appropriate exit codes
///
/// # Arguments
/// * `error` - The error to map
///
/// # Returns
/// * `i32` - Exit code (1 for general errors, 3 for Python not found, 4 for installation failures)
fn get_exit_code(error: &PackageError) -> i32 {
    match error {
        PackageError::PythonNotFound => 3,
        PackageError::InstallationFailed(_) | PackageError::UninstallationFailed(_) => 4,
        PackageError::InvalidPackageSpec(_) => 5,
        PackageError::PackageNotFound(_) => 6,
        _ => 1,
    }
}
