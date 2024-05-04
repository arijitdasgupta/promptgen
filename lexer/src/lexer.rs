#[derive(Debug, PartialEq, Eq)]
struct StringyParseResult<'a> {
    relative_end_index: usize,
    data: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LexxerError {
    InvalidSyntax,
    UnterminatedLabelLiteral,
    UnterminatedStringLiteral,
    InvalidLabelCharacter,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    RightAngular,
    LeftAngular,
    StringLiteral(&'a str),
    LabelLiteral(&'a str),
}

fn parse_label_block_greedily<'a>(data: &'a str) -> Result<StringyParseResult, LexxerError> {
    let mut idx: usize = 1;

    while true {
        let d = data.as_bytes().get(idx);
        match d {
            None => return Err(LexxerError::UnterminatedLabelLiteral),
            Some(x) if x.is_ascii_whitespace() => return Err(LexxerError::InvalidLabelCharacter),
            Some(x) if *x == "\n".as_bytes()[0] => return Err(LexxerError::InvalidLabelCharacter),
            Some(x) if *x == ")".as_bytes()[0] => break,
            _ => (),
        }
        idx = idx + 1;
    }

    Ok(StringyParseResult {
        relative_end_index: idx,
        data: &data[1..idx],
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
    cursor_position: usize,
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
                let StringyParseResult {
                    relative_end_index,
                    data,
                } = parse_string_literal_greedily(&data[self.scan_position..])?;
                self.scan_position = self.scan_position + relative_end_index;
                result.push(Token::StringLiteral(data))
            } else if (*char == "(".as_bytes()[0]) {
                let StringyParseResult {
                    relative_end_index,
                    data,
                } = parse_label_block_greedily(&data[self.scan_position..])?;
                self.scan_position = self.scan_position + relative_end_index;
                result.push(Token::LabelLiteral(data))
            }

            self.scan_position += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::{Lexxer, LexxerError, StringyParseResult};

    use super::{parse_string_literal_greedily, parse_label_block_greedily, Token};

    // A whole prompt chunk, e.g.
    // > (LABEL) \"Hello World\"
    #[test]
    fn lex_a_line() {
        let input = "> (LABEL) \"Hello World\"";

        let expected_tokens = vec![
            Token::RightAngular,
            Token::LabelLiteral("LABEL"),
            Token::StringLiteral("Hello World"),
        ];
        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // A whole answer chunk, e.g.
    // < (LABEL) \"Hello World\"
    #[test]
    fn lex_a_answer_line() {
        let input = "< (LABEL) \"Hello World\"";

        let expected_tokens = vec![
            Token::LeftAngular,
            Token::LabelLiteral("LABEL"),
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
        let input = "> (LABEL) \"Hello\nWorld\"";

        let expected_tokens = vec![
            Token::RightAngular,
            Token::LabelLiteral("LABEL"),
            Token::StringLiteral("Hello\nWorld"),
        ];
        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // Bad syntaxes
    #[test]
    fn fail_to_lex_bad_labels_1() {
        let input = "> (LABEL \"Hello\nWorld\"";

        let expected_result: Result<Vec<Token>, LexxerError> =
            Err(LexxerError::InvalidLabelCharacter);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_lex_bad_labels_2() {
        let input = "> (LABEL_BOO_BOO";

        let expected_result: Result<Vec<Token>, LexxerError> =
            Err(LexxerError::UnterminatedLabelLiteral);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_lex_bad_labels_3() {
        let input = "> (LABEL\nBOO_BOO";

        let expected_result: Result<Vec<Token>, LexxerError> =
            Err(LexxerError::InvalidLabelCharacter);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // Bad syntaxes being ignored
    #[test]
    fn ignores_weird_stuff_1() {
        let input = "> LABEL)";

        let expected_tokens = vec![Token::RightAngular];

        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn ignores_weird_stuff_2() {
        let input = "> (LABEL) Hello World";

        let expected_tokens = vec![Token::RightAngular, Token::LabelLiteral("LABEL")];

        let expected_result: Result<Vec<Token>, LexxerError> = Ok(expected_tokens);

        let mut lexxer = Lexxer::new();
        let result = lexxer.parse(input);
        assert_eq!(result, expected_result);
    }

    // label literals
    // e.g. (LABEL)
    #[test]
    fn lex_a_label() {
        let input = "(LABEL)";

        let result = parse_label_block_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 6,
            data: "LABEL",
        });

        assert_eq!(result, expected_result);
    }

    // label literals with LABELs
    // e.g. (LABEL_1)
    #[test]
    fn lex_a_label_with_symbols() {
        let input = "(LABEL_1)";

        let result = parse_label_block_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 8,
            data: "LABEL_1",
        });

        assert_eq!(result, expected_result);
    }

    // Lex a blank LABEL
    // e.g. (LABEL_1)
    #[test]
    fn lex_a_blank_label() {
        let input = "()";

        let result = parse_label_block_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 1,
            data: "",
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
            data: "Hello",
        });

        assert_eq!(result, expected_result);
    }

    // String literals
    #[test]
    fn lex_a_blank_string_literal() {
        let input = "\"\"";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> = Ok(StringyParseResult {
            relative_end_index: 1,
            data: "",
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_parse_unterminated_string_literal() {
        let input = "\"Hello";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> =
            Err(LexxerError::UnterminatedStringLiteral);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn fail_to_parse_naked_string_literal() {
        let input = "Hello World!";

        let result = parse_string_literal_greedily(input);
        let expected_result: Result<StringyParseResult, LexxerError> =
            Err(LexxerError::UnterminatedStringLiteral);

        assert_eq!(result, expected_result);
    }
}
