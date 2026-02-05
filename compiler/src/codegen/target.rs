//! Target machine and object file generation.
//!
//! This module handles LLVM target initialization and object file output.

use super::Codegen;
use super::error::{CodegenError, CodegenErrorKind};
use inkwell::OptimizationLevel;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use std::path::Path;

impl<'ctx> Codegen<'ctx> {
    /// Writes the compiled module to a native object file.
    ///
    /// This method initializes the native target (if not already done),
    /// creates a target machine for the host platform, and writes the
    /// compiled LLVM IR to an object file that can be linked.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the object file should be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to initialize the native target
    /// - Failed to create the target machine
    /// - Failed to write the object file
    ///
    /// # Platform Support
    ///
    /// This method generates object files for the host platform only.
    /// The object file format depends on the host:
    /// - macOS: Mach-O
    /// - Linux: ELF
    /// - Windows: COFF
    ///
    /// Cross-compilation is not currently supported.
    pub fn write_object_file(&self, path: &Path) -> Result<(), CodegenError> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            CodegenError::without_span(
                CodegenErrorKind::TargetError,
                format!("Failed to initialize native target: {}", e),
            )
        })?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| {
            CodegenError::without_span(
                CodegenErrorKind::TargetError,
                format!("Failed to get target for triple '{}': {}", triple, e),
            )
        })?;

        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();

        let cpu_str = cpu.to_str().map_err(|_| {
            CodegenError::without_span(
                CodegenErrorKind::TargetError,
                "CPU name contains invalid UTF-8",
            )
        })?;
        let features_str = features.to_str().map_err(|_| {
            CodegenError::without_span(
                CodegenErrorKind::TargetError,
                "CPU features contain invalid UTF-8",
            )
        })?;

        let target_machine = target
            .create_target_machine(
                &triple,
                cpu_str,
                features_str,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CodegenError::without_span(
                    CodegenErrorKind::TargetError,
                    format!(
                        "Failed to create target machine for triple '{}', CPU '{}'. \
                         This may indicate an unsupported platform or LLVM configuration issue.",
                        triple, cpu_str
                    ),
                )
            })?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| {
                CodegenError::without_span(
                    CodegenErrorKind::TargetError,
                    format!("Failed to write object file to '{}': {}", path.display(), e),
                )
            })?;

        Ok(())
    }
}
