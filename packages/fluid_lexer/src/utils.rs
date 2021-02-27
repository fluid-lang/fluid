/// Advance to the next character.
#[macro_export]
macro_rules! advance {
    ($self:ident, [$($char:tt => $ret:expr),*], $default:expr) => {{
        let tok = {
            $self.advance();

            $(
                if !$self.is_eof() && $char == $self.current_char() {
                    let token = $self.new_token($ret, $self.position, $self.position + 2);

                    $self.advance();

                    return Ok(token);
                }
            )*

            $default
        };

        let token = $self.new_token(tok, $self.position, $self.position + 1);

        return Ok(token);
    }};
    ($self:ident, $token:expr) => {{
        let token = $self.new_token($token, $self.position, $self.position + 1);

        $self.advance();

        return Ok(token);
    }};
}
