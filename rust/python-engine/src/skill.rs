//! @skill:id      ai.luminarys.rust.python-engine
//! @skill:name    "Python Engine"
//! @skill:version 1.0.0
//! @skill:desc    "Sandboxed Python 3 execution. No stdlib (import will fail). Available built-ins: print, len, range, list, dict, tuple, set, str, int, float, bool, bytes, abs, min, max, sum, round, pow, divmod, sorted, reversed, enumerate, zip, map, filter, any, all, isinstance, type, repr, hash, id, hex, oct, bin, chr, ord, format, iter, next, callable."

use luminarys_sdk::prelude::*;
use rustpython_vm::{
    builtins::PyBaseException,
    function::FuncArgs,
    AsObject, Interpreter, PyObjectRef, PyRef, VirtualMachine,
};

#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    _dest: *mut u8,
    _len: usize,
) -> Result<(), getrandom::Error> {
    Ok(())
}

// ── Skill logic ───────────────────────────────────────────────────────────────

/// Execute Python 3 code using built-ins only.
///
/// @skill:method execute "Execute Python 3 code using built-ins only. No import statements. Use print() for output. Available: print, len, range, list, dict, set, str, int, float, bool, abs, min, max, sum, round, pow, sorted, enumerate, zip, map, filter, any, all, isinstance, type, repr."
/// @skill:param  code required "Python source code to execute (no imports)"
/// @skill:result "Captured output from print() calls, or error traceback"
pub fn execute(_ctx: &mut Context, code: String) -> Result<String, SkillError> {
    let interpreter = Interpreter::without_stdlib(Default::default());

    let result: String = interpreter.enter(|vm: &VirtualMachine| {
        let scope = vm.new_scope_with_builtins();

        // Intercept print() via a native Rust closure — no sys/io needed.
        let captured = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let cap = captured.clone();

        let print_fn = vm.new_function(
            "print",
            move |args: FuncArgs, vm: &VirtualMachine| -> rustpython_vm::PyResult<PyObjectRef> {
                let parts: Vec<String> = args.args.iter()
                    .map(|a: &PyObjectRef| a.str(vm).map(|s: rustpython_vm::builtins::PyStrRef| s.to_string()).unwrap_or_default())
                    .collect();
                // sep kwarg: get_kwarg requires a default value
                let sep_obj = args.get_kwarg("sep", vm.ctx.new_str(" ").into());
                let sep: String = sep_obj.str(vm).map(|s: rustpython_vm::builtins::PyStrRef| s.to_string()).unwrap_or_else(|_| " ".to_string());
                let end_obj = args.get_kwarg("end", vm.ctx.new_str("\n").into());
                let end: String = end_obj.str(vm).map(|s: rustpython_vm::builtins::PyStrRef| s.to_string()).unwrap_or_else(|_| "\n".to_string());
                cap.borrow_mut().push_str(&parts.join(&sep));
                cap.borrow_mut().push_str(&end);
                Ok(vm.ctx.none())
            },
        );

        if let Err(e) = scope.globals.set_item("print", print_fn.into(), vm) {
            return format!("[setup error] {}", fmt_exc(vm, e));
        }

        // Run user code.
        let run_result = vm.run_block_expr(scope, &code);
        let out = captured.borrow().trim_end_matches('\n').to_string();

        match run_result {
            Ok(_) => out,
            Err(exc) => {
                let tb = fmt_exc(vm, exc);
                if out.is_empty() { tb } else { format!("{out}\n\n[Error]\n{tb}") }
            }
        }
    });

    Ok(result)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn fmt_exc(vm: &VirtualMachine, exc: PyRef<PyBaseException>) -> String {
    let mut buf = String::new();
    vm.write_exception(&mut buf, &exc).ok();
    let s = buf.trim_end().to_string();
    if s.is_empty() {
        exc.as_object()
            .repr(vm)
            .ok()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "unknown error".to_string())
    } else {
        s
    }
}
