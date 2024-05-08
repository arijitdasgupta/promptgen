use crate::lexer::Token;

#[derive(PartialEq, Eq, Debug)]
struct Response<'a> {
    text: &'a str,
    label: Option<&'a str>,
}

#[derive(PartialEq, Eq, Debug)]
struct Prompt<'a> {
    text: &'a str,
    label: Option<&'a str>,
    possible_responses: Vec<Response<'a>>,
}

#[derive(PartialEq, Eq, Debug)]
struct PromptTree<'a> {
    prompts: Vec<Prompt<'a>>,
}

struct Parser {
    scan_position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParsingError {
    InvalidSyntax,
}

type ParsedChunk<'a> = (usize, Option<&'a str>, &'a str);

/// Parsed prompt part, Label, Text or just Text
fn parse_prompt_part_greedily<'a>(tokens: &[Token<'a>]) -> Result<ParsedChunk<'a>, ParsingError> {
    let first_token = tokens.get(0);
    let second_token = tokens.get(1);

    match (first_token, second_token) {
        (Some(Token::LabelLiteral(label_text)), Some(Token::StringLiteral(string_text))) => {
            Ok((1, Some(label_text), string_text))
        }
        (Some(Token::StringLiteral(string_text)), _) => Ok((0, None, string_text)),
        _ => Err(ParsingError::InvalidSyntax),
    }
}

impl Parser {
    fn new() -> Self {
        Self { scan_position: 0 }
    }

    fn parse_tokens<'a, 'b>(
        self: &'b mut Self,
        tokens: Vec<Token<'a>>,
    ) -> Result<PromptTree<'a>, ParsingError> {
        loop {
            match tokens.get(self.scan_position) {
                Some(Token::RightAngular) => {
                    let (new_index, label, text) =
                        parse_prompt_part_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = new_index + 1;

                    // TODO: Store as question
                }
                Some(Token::LeftAngular) => {
                    let (new_index, label, text) =
                        parse_prompt_part_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = new_index + 1;

                    // TODO: Store as answer
                }
                Some(_) => {
                    return Err(ParsingError::InvalidSyntax);
                }
                None => break, // TODO
            }
        }

        Ok(PromptTree { prompts: vec![] })
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Token, parser::PromptTree};

    use super::{parse_prompt_part_greedily, Parser, ParsingError};

    #[test]
    fn parse_prompt() {
        let input_tokens = vec![];
        let expected_parse_result = PromptTree { prompts: vec![] };

        let mut parser = Parser::new();
        let parse_result = parser.parse_tokens(input_tokens);

        assert_eq!(parse_result, Ok(expected_parse_result));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_with_label() {
        let input_tokens = vec![
            Token::RightAngular,
            Token::LabelLiteral("LABEL_1"),
            Token::StringLiteral("Hello World"),
        ];

        let parse_results = parse_prompt_part_greedily(&input_tokens[1..]);
        let expected_parse_result = (1, Some("LABEL_1"), "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_without_label() {
        let input_tokens = vec![Token::LeftAngular, Token::StringLiteral("Hello World")];

        let parse_results = parse_prompt_part_greedily(&input_tokens[1..]);
        let expected_parse_result = (0, None, "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Fail to parse invalid syntax
    #[test]
    fn parse_bad_syntax() {
        let input_tokens = vec![Token::RightAngular];

        let parse_results = parse_prompt_part_greedily(&input_tokens);
        let expected_parse_result = Err(ParsingError::InvalidSyntax);
        assert_eq!(parse_results, expected_parse_result);
    }
}
