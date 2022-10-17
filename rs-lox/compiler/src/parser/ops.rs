use super::*;
impl<'a> Parser<'a> {
    pub(super) fn unary(&mut self) -> COMPError<()> {
        let op = self.prev.ty;

        self.expression(Precedence::Unary)?;

        match op {
            TokenType::Minus => self.emit_op(OpCode::NEGATE),
            TokenType::Bang => self.emit_op(OpCode::NOT),
            _ => {}
        };

        Ok(())
    }

    pub(super) fn binary(&mut self) -> COMPError<()> {
        // at this point we already have the left hand side expression result on the stack
        // now we need to figure out what should we parse next. get that expression, push it on the stack
        // and do the binary operation on both the values in the stack.

        // the way bob does that in C is to have parse rules associated with every op token.
        // i can gothe same way as bob did maybe i should star with that and see how i can improve from there
        // I think it's much easir to do that with some pattern matching or something inplace. no need for extra functions here
        let op = self.cur.ty;
        self.move_to_next_token();

        match op {
            // do i need some kind of jump?
            TokenType::And => self.and_(op.into())?,
            TokenType::Or => self.or_(op.into())?,
            _ => self.expression(op.into())?,
        }

        // apply the binary op on both expressions
        match op {
            TokenType::Plus => self.emit_op(OpCode::ADD),
            TokenType::Minus => self.emit_op(OpCode::SUB),
            TokenType::Star => self.emit_op(OpCode::MUL),
            TokenType::Slash => self.emit_op(OpCode::DIV),
            TokenType::EqualEqual => self.emit_op(OpCode::EQUAL),
            TokenType::BangEqual => {
                self.emit_op(OpCode::EQUAL);
                self.emit_op(OpCode::NOT);
            }
            TokenType::Greater => self.emit_op(OpCode::GREATER),
            TokenType::GreaterEqual => {
                self.emit_op(OpCode::LESS);
                self.emit_op(OpCode::NOT);
            }
            TokenType::Less => self.emit_op(OpCode::LESS),
            TokenType::LessEqual => {
                self.emit_op(OpCode::GREATER);
                self.emit_op(OpCode::NOT)
            }
            TokenType::And | TokenType::Or => {
                // handled in previous match
            }
            _ => {
                dbg!(self.prev, self.cur);
                todo!()
            }
        }

        // let rulle = get_parse_rule(op);
        Ok(())
    }

    fn and_(&mut self, prec: Precedence) -> COMPError<()> {
        let after_fst_expr_ip = self.chunk.count();
        self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
        self.expression(prec)?;
        self.emit_op(OpCode::AND);
        let after_snd_expr_ip = self.chunk.count();
        self.chunk.patch_op(
            OpCode::JUMP_IF_FALSE(after_snd_expr_ip as InstructAddr),
            after_fst_expr_ip,
        );
        Ok(())
    }

    fn or_(&mut self, prec: Precedence) -> COMPError<()> {
        let after_fst_expr_ip = self.chunk.count();
        self.emit_op(OpCode::JUMP_IF_FALSE(0xFFFF));
        self.expression(prec)?;
        self.emit_op(OpCode::AND);
        let after_snd_expr_ip = self.chunk.count();
        self.chunk.patch_op(
            OpCode::JUMP_IF_FALSE(after_snd_expr_ip as InstructAddr),
            after_fst_expr_ip,
        );
        Ok(())
    }
}
