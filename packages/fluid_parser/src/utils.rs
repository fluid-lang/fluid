/// Advance to the next token.
#[macro_export]
macro_rules! advance {
    ($self:ident) => {{
        $self.index += 1;
    }};

    ($self:ident, $tok:expr) => {
        if $tok == *$self.peek() {
            $self.index += 1;
        } else {
            // TODO: Implement error system.
            todo!("Expected {:?}. found {:?}", $tok, $self.peek());
        }
    };

    ($self:ident => $tok:path) => {
        match $self.peek().clone() {
            $tok(en) => {
                $self.index += 1;

                en
            }
            _ => {
                $self.index += 1;

                // TODO: Implement error system.
                todo!("Implement error system.");
            }
        }
    };
}
