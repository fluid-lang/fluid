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
                if !$self.is_eof() && $char == $self.current_char() {
                    let token = $self.new_token($ret, $self.index, $self.index + 2);

                    advance!($self);

                    return Ok(token);
                }
            )*

            $default
        };

        let token = $self.new_token(tok, $self.index, $self.index + 1);

        return Ok(token);
    }};
    ($self:ident, $token:expr) => {{
        let token = $self.new_token($token, $self.index, $self.index + 1);

        advance!($self);

        return Ok(token);
    }};
}
