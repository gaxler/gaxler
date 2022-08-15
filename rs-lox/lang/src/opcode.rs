
pub type ConstIdx = u8;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OpCode {
    RETURN,
    CONSTANT(ConstIdx), // load the constant to the vm for use
    NEGATE,
    NOT,
    NIL,
    TRUE,

    FALSE,
    EQUAL,
    LESS,
    GREATER,
    ADD,
    SUB,
    MUL,
    DIV,

    PRINT,
    POP,
    DEFINE_GLOBAL(ConstIdx),
    GET_GLOBAL(ConstIdx),
    SET_GLOBAL(ConstIdx),
}

