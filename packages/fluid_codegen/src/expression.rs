use fluid_mangle::mangle_function_name;
use fluid_parser::{BinaryOp, Expression, Literal, Type, UnaryOp};

use llvm::core::*;

use crate::{cstring, utils::FluidValueRef, CodeGen};

impl CodeGen {
    /// Generate an expression.
    pub(crate) unsafe fn gen_expression(&mut self, expression: &Expression) -> FluidValueRef {
        match expression {
            Expression::Literal(ref literal) => self.gen_literal(literal),
            Expression::VarRef(ref name) => self.gen_var_ref(name),
            Expression::FunctionCall(ref name, ref args) => self.gen_function_call(name, args),
            Expression::BinaryOp(ref lhs, ref op, ref rhs) => self.gen_binary(lhs, op, rhs),
            Expression::Unary(ref op, ref rhs) => self.gen_unary(op, rhs),
            _ => unimplemented!(),
        }
    }

    /// Generate a unary expression.
    pub(crate) unsafe fn gen_unary(&mut self, op: &UnaryOp, rhs: &Expression) -> FluidValueRef {
        let rhs = self.gen_expression(rhs);

        match op {
            UnaryOp::Neg => FluidValueRef::new(rhs.kind, LLVMBuildNeg(self.builder, rhs.value, cstring!("nottmp").as_ptr())),
            UnaryOp::Not => {
                unimplemented!()
            }
        }
    }

    /// Generate a binary expression.
    pub(crate) unsafe fn gen_binary(&mut self, lhs: &Expression, op: &BinaryOp, rhs: &Expression) -> FluidValueRef {
        let lhs = self.gen_expression(lhs);
        let rhs = self.gen_expression(rhs);

        let res = match op {
            BinaryOp::Add => {
                if lhs.kind == Type::Number {
                    LLVMBuildAdd(self.builder, lhs.value, rhs.value, cstring!("addtmp").as_ptr())
                } else {
                    LLVMBuildFAdd(self.builder, lhs.value, rhs.value, cstring!("addtmp").as_ptr())
                }
            }
            BinaryOp::Subtract => {
                if lhs.kind == Type::Number {
                    LLVMBuildSub(self.builder, lhs.value, rhs.value, cstring!("subtmp").as_ptr())
                } else {
                    LLVMBuildFSub(self.builder, lhs.value, rhs.value, cstring!("subtmp").as_ptr())
                }
            }
            BinaryOp::Mul => {
                if lhs.kind == Type::Number {
                    LLVMBuildMul(self.builder, lhs.value, rhs.value, cstring!("multmp").as_ptr())
                } else {
                    LLVMBuildFMul(self.builder, lhs.value, rhs.value, cstring!("multmp").as_ptr())
                }
            }
            _ => unimplemented!(),
        };

        FluidValueRef::new(lhs.kind, res)
    }

    /// Generate a variable reference.
    pub(crate) unsafe fn gen_var_ref(&mut self, var_name: &str) -> FluidValueRef {
        let var = self.symbol_table.get_variable(var_name).unwrap();

        assert!(var.initialized);

        FluidValueRef::new(var.kind, LLVMBuildLoad(self.builder, var.alloca, cstring!("{}", var_name).as_ptr()))
    }

    /// Generate an literal.
    pub(crate) unsafe fn gen_literal(&mut self, literal: &Literal) -> FluidValueRef {
        match literal {
            Literal::Number(ref number) => self.gen_number_literal(*number),
            Literal::Bool(ref bool) => self.gen_bool_literal(*bool),
            _ => unimplemented!(),
        }
    }

    /// Generate a function call.
    pub(crate) unsafe fn gen_function_call(&mut self, name: &str, args: &Vec<Expression>) -> FluidValueRef {
        let mut cargs = vec![];

        for arg in args {
            let arg = self.gen_expression(arg);

            cargs.push(arg);
        }

        let func_name = mangle_function_name(name.into(), cargs.iter().map(|fref| fref.kind).collect::<Vec<_>>());
        let func = self.symbol_table.get_function(&func_name);
        let func = match func {
            Some(func) => func,
            None => self.symbol_table.current_scope_parent().get_function(&func_name).unwrap(),
        };

        let value = LLVMBuildCall(
            self.builder,
            func.value,
            cargs.iter().map(|arg| arg.value).collect::<Vec<_>>().as_mut_ptr(),
            cargs.len() as u32,
            cstring!("").as_ptr(),
        );

        FluidValueRef::new(func.return_type, value)
    }

    /// Generate an number literal.
    #[inline]
    pub(crate) unsafe fn gen_number_literal(&mut self, number: u64) -> FluidValueRef {
        FluidValueRef::new(Type::Number, LLVMConstInt(LLVMInt64TypeInContext(self.context), number, 0))
    }

    /// Generate an boolean literal.
    #[inline]
    pub(crate) unsafe fn gen_bool_literal(&mut self, bool: bool) -> FluidValueRef {
        let value = if bool { 1 } else { 0 };

        FluidValueRef::new(Type::Bool, LLVMConstInt(LLVMInt1TypeInContext(self.context), value, 0))
    }
}
