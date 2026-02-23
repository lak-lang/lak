//! Variable binding management for code generation.
//!
//! This module defines [`VarBinding`], which represents a variable's stack
//! allocation and type information during code generation.

use super::error::CodegenError;
use crate::ast::Type;
use crate::token::Span;
use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;

/// A variable binding in the symbol table.
///
/// Stores the stack allocation pointer and declared type for a variable,
/// enabling variable lookups and type checking during code generation.
///
/// # Invariants
///
/// The LLVM type of `alloca` must correspond to `ty`:
/// - `Type::I8` → `alloca` points to an LLVM `i8`
/// - `Type::I16` → `alloca` points to an LLVM `i16`
/// - `Type::I32` → `alloca` points to an LLVM `i32`
/// - `Type::I64` → `alloca` points to an LLVM `i64`
/// - `Type::U8` → `alloca` points to an LLVM `i8`
/// - `Type::U16` → `alloca` points to an LLVM `i16`
/// - `Type::U32` → `alloca` points to an LLVM `i32`
/// - `Type::U64` → `alloca` points to an LLVM `i64`
/// - `Type::Bool` → `alloca` points to an LLVM `i1`
/// - `Type::String` → `alloca` points to an LLVM `ptr` (pointer to string data)
///
/// This invariant is enforced by creating bindings only through
/// [`VarBinding::new`], which allocates the correct LLVM type.
#[derive(Clone, Debug)]
pub(super) struct VarBinding<'ctx> {
    /// The stack allocation for this variable.
    alloca: PointerValue<'ctx>,
    /// The declared type of this variable.
    ty: Type,
}

impl<'ctx> VarBinding<'ctx> {
    /// Creates a new variable binding with a stack allocation.
    ///
    /// This constructor ensures the invariant that `alloca`'s LLVM type
    /// matches the declared `ty`.
    ///
    /// # Arguments
    ///
    /// * `builder` - The LLVM IR builder
    /// * `context` - The LLVM context
    /// * `ty` - The Lak type for this variable
    /// * `name` - The variable name (used for LLVM IR naming)
    /// * `span` - The source span for error reporting
    ///
    /// # Returns
    ///
    /// * `Ok(VarBinding)` - A new binding with a correctly-typed stack allocation.
    /// * `Err(CodegenError)` - If LLVM fails to create the alloca instruction.
    pub(super) fn new(
        builder: &inkwell::builder::Builder<'ctx>,
        context: &'ctx Context,
        ty: &Type,
        name: &str,
        span: Span,
    ) -> Result<Self, CodegenError> {
        let llvm_type: BasicTypeEnum = match ty {
            Type::I8 => context.i8_type().into(),
            Type::I16 => context.i16_type().into(),
            Type::I32 => context.i32_type().into(),
            Type::I64 => context.i64_type().into(),
            Type::U8 => context.i8_type().into(),
            Type::U16 => context.i16_type().into(),
            Type::U32 => context.i32_type().into(),
            Type::U64 => context.i64_type().into(),
            Type::F32 => context.f32_type().into(),
            Type::F64 => context.f64_type().into(),
            Type::String => context.ptr_type(AddressSpace::default()).into(),
            Type::Bool => context.bool_type().into(),
        };
        let alloca = builder.build_alloca(llvm_type, name).map_err(|e| {
            CodegenError::internal_variable_alloca_failed(name, &e.to_string(), span)
        })?;
        Ok(VarBinding {
            alloca,
            ty: ty.clone(),
        })
    }

    /// Returns the stack allocation pointer for this variable.
    pub(super) fn alloca(&self) -> PointerValue<'ctx> {
        self.alloca
    }

    /// Returns the declared type of this variable.
    pub(super) fn ty(&self) -> &Type {
        &self.ty
    }
}
