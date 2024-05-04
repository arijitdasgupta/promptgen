#[derive(Debug, PartialEq, Eq)]
struct StringyParseResult<'a> {
    relative_end_index: usize,
    data: &'a str
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LexxerError {
    InvalidSyntax,
    UnterminatedSymbolLiteral,
    UnterminatedStringLiteral,
    InvalidSymbolCharacter
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    RightAngular,
    LeftAngular,
    StringLiteral(&'a str),
    Symbol(&'a str)
}

fn parse_symbol_block_greedily<'a>(data: &'a str) -> Result<StringyParseResult, LexxerError> {
    let mut idx: usize = 1;

    while true {
        let d = data.as_bytes().get(idx);
        match d {
            None => return Err(LexxerError::UnterminatedSymbolLiteral),
            Some(x) if x.is_ascii_whitespace() => return Err(LexxerError::InvalidSymbolCharacter),
            Some(x) if *x == "\n".as_bytes()[0] => return Err(LexxerError::InvalidSymbolCharacter),
            Some(x) if *x == ")".as_bytes()[0] => break,
            _ => (),
        }
        idx = idx + 1;
    }

    Ok(StringyParseResult {
        relative_end_index: idx,
        data: &data[1..idx]
    })
}

fn parse_string_literal_greedily<'a>(data: &'a str) -> Result<StringyParseResult, LexxerError> {
    let mut idx: usize = 1;

    while true {
        let d = data.as_bytes().get(idx);
        match d {
            None => return Err(LexxerError::UnterminatedStringLiteral),
            Some(x) if *x == "\"".as_bytes()[0] => break,
            _ => (),
        }

        idx = idx + 1;
    }

    Ok(StringyParseResult {
        relative_end_index: idx,
        data: &data[1..idx],
    })
}

struct Lexer {
    cursor_position: usize
}

struct Lexxer {
    scan_position: usize,
}

impl Lexxer {
    fn new() -> Self {
        Lexxer { scan_position: 0 }
    }

    fn parse<'a, 'b>(self: &'b mut Self, data: &'a str) -> Result<Vec<Token<'a>>, LexxerError> {
        let mut result: Vec<Token<'a>> = vec![];

        while let Some(char) = data.as_bytes().get(self.scan_position) {
            if (*char == ">".as_bytes()[0]) {
                result.push(Token::RightAngular);
            } else if (*char == "<".as_bytes()[0]) {
                result.push(Token::LeftAngular);
            } else if (*char == "\"".as_bytes()[0]) {
                let StringyParseResult { relative_end_index, data } = parse_string_literal_greedily(&data[self.scan_position..])?;
                self.scan_position = self.scan_position + relative_end_index;
                result.push(Token::StringLiteral(data))
            } else if (*char == "(".as_bytes()[0]) {
                let StringyParseResult { relative_end_index, data } = parse_symbol_block_greedily(&data[self.scan_position..])?;
                self.scan_position = self.scan_position + relative_end_index;
                result.push(Token::Symbol(data))
            }

            self.scan_position += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::{Lexxer, LexxerError, StringyParseResult};

    use super::{parse_string_literal_greedily, parse_symbol_block_greedily, Token};

    // A whole prompt chunk, e.g.
    // > (SYMBOL) \"Hello World\"
    #[test]
    fn lex_a_line() {
        let input = "> (SYMBOL) \"Hello World\"";

        let expected_tokens = vec![
            Token::RightAngular,
            Token::Symbol("SYMBOL"),
            Token::StringLiteral("Hello World"),
        ];
        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);
        
        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // A whole multiline prompt chunk
    #[test]
    fn lex_multi_line_string() {
        let input = "> (SYMBOL) \"Hello\nWorld\"";

        let expected_tokens = vec![
            Token::RightAngular,
            Token::Symbol("SYMBOL"),
            Token::StringLiteral("Hello\nWorld"),
        ];
        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);
        
        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // Bad syntaxes
    #[test]
    fn fail_to_lex_bad_symbols_1() {
        let input = "> (SYMBOL \"Hello\nWorld\"";

        let expected_result: Result<Vec<Token>, LexxerError> = Err(LexxerError::InvalidSymbolCharacter);
        
        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_lex_bad_symbols_2() {
        let input = "> (SYMBOL_BOO_BOO";

        let expected_result: Result<Vec<Token>, LexxerError> = Err(LexxerError::UnterminatedSymbolLiteral);
        
        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_lex_bad_symbols_3() {
        let input = "> (SYMBOL\nBOO_BOO";

        let expected_result: Result<Vec<Token>, LexxerError> = Err(LexxerError::InvalidSymbolCharacter);
        
        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // Symbol literals
    // e.g. (SYMBOL)
    #[test]
    fn lex_a_symbol() {
        let input = "(SYMBOL)";

        let result = parse_symbol_block_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 7,
            data: "SYMBOL"
        });

        assert_eq!(result, expected_result);
    }

    // String literals
    #[test]
    fn lex_a_string_literal() {
        let input = "\"Hello\"";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 6,
            data: "Hello"
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_parse_unterminated_string_literal() {
        let input = "\"Hello";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Err(LexxerError::UnterminatedStringLiteral);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_parse_naked_string_literal() {
        let input = "Hello World!";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Err(LexxerError::UnterminatedStringLiteral);

        assert_eq!(result, expected_result);
    }
}