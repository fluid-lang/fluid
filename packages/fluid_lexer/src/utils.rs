/// Advance to the next character.
#[macro_export]
macro_rules! advance {
    ($self:ident) => {{
        $self.index += 1;
    }};
    ($self:ident, [$($char:tt => $ret:expr),*], $default:expr) => {{
        let tok = {
            advance!($self);

            $(
                if $self.peek().is_some() && $char == $self.current_char() {
                    let token = $self.new_token($ret);

                    advance!($self);

                    return Ok(token);
                }
            )*

            $default
        };

        let token = $self.new_token(tok);

        return Ok(token);
    }};
    ($self:ident, $token:expr) => {{
        let token = $self.new_token($token);

        advance!($self);

        return Ok(token);
    }};
}
