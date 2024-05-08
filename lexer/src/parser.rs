use crate::lexer::Token;

#[derive(PartialEq, Eq, Debug)]
enum ChunkVariant {
    Prompt,
    Response,
}

#[derive(PartialEq, Eq, Debug)]
struct Chunk<'a> {
    variant: ChunkVariant,
    text: &'a str,
    label: Option<&'a str>,
}

struct Parser {
    scan_position: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParsingError {
    InvalidSyntax,
}

type ParsedTextAndLabel<'a> = (usize, Option<&'a str>, &'a str);

/// Parsed prompt part, Label, Text or just Text
fn parse_label_and_text_greedily<'a>(
    tokens: &[Token<'a>],
) -> Result<ParsedTextAndLabel<'a>, ParsingError> {
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
    ) -> Result<Vec<Chunk<'a>>, ParsingError> {
        let mut chunks: Vec<Chunk<'a>> = vec![];

        loop {
            match tokens.get(self.scan_position) {
                Some(Token::RightAngular) => {
                    let (new_index, label, text) =
                        parse_label_and_text_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = new_index + 1;
                    chunks.push(Chunk {
                        variant: ChunkVariant::Prompt,
                        text,
                        label,
                    });
                }
                Some(Token::LeftAngular) => {
                    let (new_index, label, text) =
                        parse_label_and_text_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = new_index + 1;

                    chunks.push(Chunk {
                        variant: ChunkVariant::Response,
                        text,
                        label,
                    })
                }
                Some(_) => {
                    return Err(ParsingError::InvalidSyntax);
                }
                None => break, // TODO
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Token, parser::Chunk};

    use super::{parse_label_and_text_greedily, ChunkVariant, Parser, ParsingError};

    // Parse a series of tokens into chunks
    #[test]
    fn parse_tokens_to_chunks() {
        let input_tokens = vec![
            Token::RightAngular,
            Token::StringLiteral("Hello world"),
            Token::LeftAngular,
            Token::LabelLiteral("LABEL_1"),
            Token::StringLiteral("Hello me"),
        ];

        let mut parser = Parser::new();
        let parsing_results = parser.parse_tokens(input_tokens);

        let expect_chunks = vec![
            Chunk {
                variant: ChunkVariant::Prompt,
                label: None,
                text: "Hello world",
            },
            Chunk {
                variant: ChunkVariant::Response,
                label: Some("LABEL_1"),
                text: "Hello me",
            },
        ];

        assert_eq!(parsing_results, Ok(expect_chunks));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_with_label() {
        let input_tokens = vec![
            Token::RightAngular,
            Token::LabelLiteral("LABEL_1"),
            Token::StringLiteral("Hello World"),
        ];

        let parse_results = parse_label_and_text_greedily(&input_tokens[1..]);
        let expected_parse_result = (1, Some("LABEL_1"), "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_without_label() {
        let input_tokens = vec![Token::LeftAngular, Token::StringLiteral("Hello World")];

        let parse_results = parse_label_and_text_greedily(&input_tokens[1..]);
        let expected_parse_result = (0, None, "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Fail to parse invalid syntax
    #[test]
    fn parse_bad_syntax() {
        let input_tokens = vec![Token::RightAngular];

        let parse_results = parse_label_and_text_greedily(&input_tokens);
        let expected_parse_result = Err(ParsingError::InvalidSyntax);
        assert_eq!(parse_results, expected_parse_result);
    }
}
