use super::*;

use super::utils::{stack_pop, stack_push};

pub(super) fn exec_unary(op: OpCode, stack: &mut Stack) -> RTError<()> {
    use OpCode::*;

    let unary_inp = stack_pop(stack)?;

    let unary_result = match op {
        NEGATE => match unary_inp {
            Value::Int(v) => Value::Int(-v),
            Value::Float(v) => Value::Float(-v),
            Value::Nil => Value::Nil,
            _ => return Err(RuntimeError::IllegalUnaryOp(op, unary_inp)),
        },
        // Take a value out of the stack, and negate it.
        //  is defined on the value enum that should be
        NOT => match unary_inp {
            Value::Bool(b) => Value::Bool(!b),
            Value::Nil => Value::Bool(true),
            _ => return Err(RuntimeError::IllegalUnaryOp(op, unary_inp)),
        },
        _ => unreachable!("Not a unary op!!"),
    };

    stack_push(unary_result, stack)?;

    Ok(())
}

pub(super) fn exec_binary(op: OpCode, stack: &mut Stack) -> Result<(), RuntimeError> {
    use OpCode::*;
    let v2 = stack_pop(stack)?;
    let v1 = stack_pop(stack)?;

    // clone those is cheap
    let dbg_vals = (v1.clone(), v2.clone());
    let res = match op {
        ADD => v1.add(v2),
        SUB => v1.sub(v2),
        MUL => v1.mul(v2),
        DIV => v1.div(v2),
        EQUAL => v1.eq(v2),
        GREATER => v1.greater(v2),
        LESS => v2.greater(v1),
        AND => v1.and(v2),
        OR => v1.or(v2),
        _ => unreachable!("Non Binary!!"),
    };

    if let Value::Nil = res {
        let (v1, v2) = dbg_vals;
        let dbg_v1 = format!("{:?}", v1);
        let dbg_v2 = format!("{:?}", v2);
        return Err(RuntimeError::IllegalOp(op, dbg_v1, dbg_v2));
    }

    stack
        .push(res)
        .map_err(|e| RuntimeError::StackError(format!("{}", e)))?;
    Ok(())
}
