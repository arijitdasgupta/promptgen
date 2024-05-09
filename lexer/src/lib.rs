use chunker::ChunkingError;
use lexer::LexxerError;
use parser::{ParserError, Prompt};

mod chunker;
mod lexer;
mod parser;

#[derive(Debug)]
pub enum PromptgenErr {
    InvalidSyntax,
}

impl Into<PromptgenErr> for LexxerError {
    fn into(self) -> PromptgenErr {
        PromptgenErr::InvalidSyntax
    }
}

impl Into<PromptgenErr> for ChunkingError {
    fn into(self) -> PromptgenErr {
        PromptgenErr::InvalidSyntax
    }
}

impl Into<PromptgenErr> for ParserError {
    fn into(self) -> PromptgenErr {
        PromptgenErr::InvalidSyntax
    }
}

pub fn parse<'a>(data: &'a str) -> Result<Vec<Prompt<'a>>, PromptgenErr> {
    let mut lexer = lexer::Lexxer::new();
    let mut chunker = chunker::Chunker::new();
    let mut parser = parser::Parser::new();

    let lexed_result = lexer.parse(data).map_err(|e| e.into())?;
    let chunked_result = chunker.parse_tokens(lexed_result).map_err(|e| e.into())?;
    let parsed_result = parser.parse_chunks(chunked_result);

    Ok(parsed_result)
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::{
        parse,
        parser::{Prompt, Response},
    };

    #[test]
    fn parse_a_bit_of_text() {
        let data = read_to_string("./sample_prompts.txt").unwrap();
        let result = parse(&data).unwrap();
        let expected_results = vec![
            Prompt {
                text: "Are you a human?",
                label: Some("NO"),
                responses: vec![
                    Response {
                        text: "Yes, I am",
                        label: Some("YES"),
                    },
                    Response {
                        text: "No",
                        label: Some("ANS_NO"),
                    },
                ],
            },
            Prompt {
                text: "That's very weird! Care to try again?",
                label: Some("ANS_NO"),
                responses: vec![Response {
                    text: "Please!",
                    label: Some("NO"),
                }],
            },
            Prompt {
                text: "Nice! Glad to meet you human!",
                label: Some("YES"),
                responses: vec![],
            },
        ];

        assert_eq!(result, expected_results);
    }
}
