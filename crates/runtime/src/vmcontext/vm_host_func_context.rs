//! Definition of `VM*Context` variant for host functions.
//!
//! Keep in sync with `wasmtime_environ::VMHostFuncOffsets`.

use crate::VMFuncRef;

use super::VMOpaqueContext;
use std::any::Any;
use wasmtime_environ::{VM_ARRAY_CALL_HOST_FUNC_MAGIC, VM_NATIVE_CALL_HOST_FUNC_MAGIC};

/// The `VM*Context` for array-call host functions.
///
/// Its `magic` field must always be
/// `wasmtime_environ::VM_ARRAY_CALL_HOST_FUNC_MAGIC`, and this is how you can
/// determine whether a `VM*Context` is a `VMArrayCallHostFuncContext` versus a
/// different kind of context.
#[repr(C)]
pub struct VMArrayCallHostFuncContext {
    magic: u32,
    // _padding: u32, // (on 64-bit systems)
    pub(crate) func_ref: VMFuncRef,
    host_state: Box<dyn Any + Send + Sync>,
}

// Declare that this type is send/sync, it's the responsibility of
// `VMHostFuncContext::new` callers to uphold this guarantee.
unsafe impl Send for VMArrayCallHostFuncContext {}
unsafe impl Sync for VMArrayCallHostFuncContext {}

impl VMArrayCallHostFuncContext {
    /// Create the context for the given host function.
    ///
    /// # Safety
    ///
    /// The `host_func` must be a pointer to a host (not Wasm) function and it
    /// must be `Send` and `Sync`.
    pub unsafe fn new(
        func_ref: VMFuncRef,
        host_state: Box<dyn Any + Send + Sync>,
    ) -> Box<VMArrayCallHostFuncContext> {
        debug_assert!(func_ref.vmctx.is_null());
        let mut ctx = Box::new(VMArrayCallHostFuncContext {
            magic: wasmtime_environ::VM_ARRAY_CALL_HOST_FUNC_MAGIC,
            func_ref,
            host_state,
        });
        ctx.func_ref.vmctx = VMOpaqueContext::from_vm_array_call_host_func_context(&mut *ctx);
        ctx
    }

    /// Get the host state for this host function context.
    #[inline]
    pub fn host_state(&self) -> &(dyn Any + Send + Sync) {
        &*self.host_state
    }

    /// Get this context's `VMFuncRef`.
    #[inline]
    pub fn func_ref(&self) -> &VMFuncRef {
        &self.func_ref
    }

    /// Helper function to cast between context types using a debug assertion to
    /// protect against some mistakes.
    #[inline]
    pub unsafe fn from_opaque(opaque: *mut VMOpaqueContext) -> *mut VMArrayCallHostFuncContext {
        // See comments in `VMContext::from_opaque` for this debug assert
        debug_assert_eq!((*opaque).magic, VM_ARRAY_CALL_HOST_FUNC_MAGIC);
        opaque.cast()
    }
}

/// The `VM*Context` for native-call host functions.
///
/// Its `magic` field must always be
/// `wasmtime_environ::VM_NATIVE_CALL_HOST_FUNC_MAGIC`, and this is how you can
/// determine whether a `VM*Context` is a `VMNativeCallHostFuncContext` versus a
/// different kind of context.
#[repr(C)]
pub struct VMNativeCallHostFuncContext {
    magic: u32,
    // _padding: u32, // (on 64-bit systems)
    func_ref: VMFuncRef,
    host_state: Box<dyn Any + Send + Sync>,
}

#[test]
fn vmnative_call_host_func_context_offsets() {
    use memoffset::offset_of;
    use wasmtime_environ::{HostPtr, PtrSize};
    assert_eq!(
        usize::from(HostPtr.vmnative_call_host_func_context_func_ref()),
        offset_of!(VMNativeCallHostFuncContext, func_ref)
    );
}

// Declare that this type is send/sync, it's the responsibility of
// `VMHostFuncContext::new` callers to uphold this guarantee.
unsafe impl Send for VMNativeCallHostFuncContext {}
unsafe impl Sync for VMNativeCallHostFuncContext {}

impl VMNativeCallHostFuncContext {
    /// Create the context for the given host function.
    ///
    /// # Safety
    ///
    /// The `host_func` must be a pointer to a host (not Wasm) function and it
    /// must be `Send` and `Sync`.
    pub unsafe fn new(
        func_ref: VMFuncRef,
        host_state: Box<dyn Any + Send + Sync>,
    ) -> Box<VMNativeCallHostFuncContext> {
        let mut ctx = Box::new(VMNativeCallHostFuncContext {
            magic: wasmtime_environ::VM_NATIVE_CALL_HOST_FUNC_MAGIC,
            func_ref,
            host_state,
        });
        ctx.func_ref.vmctx = VMOpaqueContext::from_vm_native_call_host_func_context(&mut *ctx);
        ctx
    }

    /// Get the host state for this host function context.
    #[inline]
    pub fn host_state(&self) -> &(dyn Any + Send + Sync) {
        &*self.host_state
    }

    /// Get this context's `VMFuncRef`.
    #[inline]
    pub fn func_ref(&self) -> &VMFuncRef {
        &self.func_ref
    }

    /// Helper function to cast between context types using a debug assertion to
    /// protect against some mistakes.
    #[inline]
    pub unsafe fn from_opaque(opaque: *mut VMOpaqueContext) -> *mut VMNativeCallHostFuncContext {
        // See comments in `VMContext::from_opaque` for this debug assert
        debug_assert_eq!((*opaque).magic, VM_NATIVE_CALL_HOST_FUNC_MAGIC);
        opaque.cast()
    }
}
