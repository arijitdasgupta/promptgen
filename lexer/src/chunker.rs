use crate::lexer::Token;

#[derive(PartialEq, Eq, Debug)]
pub(super) enum ChunkVariant {
    Prompt,
    Response,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Chunk<'a> {
    pub(crate) variant: ChunkVariant,
    pub(crate) text: &'a str,
    pub(crate) label: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ChunkingError {
    InvalidSyntax,
}

type ParsedTextAndLabel<'a> = (usize, Option<&'a str>, &'a str);

/// Parsed prompt part, Label, Text or just Text
fn parse_label_and_text_greedily<'a>(
    tokens: &[Token<'a>],
) -> Result<ParsedTextAndLabel<'a>, ChunkingError> {
    let first_token = tokens.get(0);
    let second_token = tokens.get(1);

    match (first_token, second_token) {
        (Some(Token::LabelLiteral(label_text)), Some(Token::StringLiteral(string_text))) => {
            Ok((1, Some(label_text), string_text))
        }
        (Some(Token::StringLiteral(string_text)), _) => Ok((0, None, string_text)),
        _ => Err(ChunkingError::InvalidSyntax),
    }
}

pub struct Chunker {
    scan_position: usize,
}

impl Chunker {
    pub fn new() -> Self {
        Self { scan_position: 0 }
    }

    pub fn parse_tokens<'a, 'b>(
        self: &'b mut Self,
        tokens: Vec<Token<'a>>,
    ) -> Result<Vec<Chunk<'a>>, ChunkingError> {
        let mut chunks: Vec<Chunk<'a>> = vec![];

        loop {
            match tokens.get(self.scan_position) {
                Some(Token::RightAngular) => {
                    let (relative_end_index, label, text) =
                        parse_label_and_text_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = self.scan_position + relative_end_index + 2; // Because we
                                                                                      // are starting scanning from a extra one
                    chunks.push(Chunk {
                        variant: ChunkVariant::Prompt,
                        text,
                        label,
                    });
                }
                Some(Token::LeftAngular) => {
                    let (relative_end_index, label, text) =
                        parse_label_and_text_greedily(&tokens[(self.scan_position + 1)..])?;
                    self.scan_position = self.scan_position + relative_end_index + 2;

                    chunks.push(Chunk {
                        variant: ChunkVariant::Response,
                        text,
                        label,
                    })
                }
                Some(_) => {
                    return Err(ChunkingError::InvalidSyntax);
                }
                None => break, // TODO
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod test {
    use crate::{chunker::Chunk, lexer::Token};

    use super::{parse_label_and_text_greedily, ChunkVariant, Chunker, ChunkingError};

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

        let mut chunker = Chunker::new();
        let parsing_results = chunker.parse_tokens(input_tokens);

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

    // Fail to parse a series of invalid tokens
    #[test]
    fn fail_to_parse_token_chunks() {
        let input_tokens = vec![
            Token::RightAngular,
            Token::LeftAngular,
            Token::StringLiteral("something"),
        ];

        let mut chunker = Chunker::new();
        let parsing_result = chunker.parse_tokens(input_tokens);

        assert_eq!(parsing_result, Err(ChunkingError::InvalidSyntax));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_with_label() {
        let input_tokens = vec![
            Token::RightAngular,
            Token::LabelLiteral("LABEL_1"),
            Token::StringLiteral("Hello World"),
            Token::LeftAngular,
        ];

        let parse_results = parse_label_and_text_greedily(&input_tokens[1..]);
        let expected_parse_result = (1, Some("LABEL_1"), "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Parse prompt tokens
    #[test]
    fn parse_prompt_part_without_label() {
        let input_tokens = vec![
            Token::LeftAngular,
            Token::StringLiteral("Hello World"),
            Token::RightAngular,
        ];

        let parse_results = parse_label_and_text_greedily(&input_tokens[1..]);
        let expected_parse_result = (0, None, "Hello World");

        assert_eq!(parse_results, Ok(expected_parse_result));
    }

    // Fail to parse invalid syntax
    #[test]
    fn parse_bad_syntax() {
        let input_tokens = vec![Token::RightAngular];

        let parse_results = parse_label_and_text_greedily(&input_tokens);
        let expected_parse_result = Err(ChunkingError::InvalidSyntax);
        assert_eq!(parse_results, expected_parse_result);
    }
}
