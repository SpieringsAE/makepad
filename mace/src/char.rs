pub trait CharExt {
    fn is_identifier_start(self) -> bool;

    fn is_identifier_continue(self) -> bool;

    fn is_open_delimiter(self) -> bool;

    fn is_close_delimiter(self) -> bool;
}

impl CharExt for char {
    fn is_identifier_start(self) -> bool {
        match self {
            'A'..='Z' | '_' | 'a'..='z' => true,
            _ => false,
        }
    }

    fn is_identifier_continue(self) -> bool {
        match self {
            '0'..='9' | 'A'..='Z' | '_' | 'a'..='z' => true,
            _ => false,
        }
    }

    fn is_open_delimiter(self) -> bool {
        match self {
            '(' | '[' | '{' => true,
            _ => false,
        }
    }

    fn is_close_delimiter(self) -> bool {
        match self {
            ')' | ']' | '}' => true,
            _ => false,
        }
    }
}
