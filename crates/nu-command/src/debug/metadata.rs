use nu_engine::CallExt;
use nu_protocol::ast::{Call, Expr, Expression};
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    record, Category, DataSource, Example, IntoPipelineData, PipelineData, PipelineMetadata,
    Record, ShellError, Signature, Span, SyntaxShape, Type, Value,
};

#[derive(Clone)]
pub struct Metadata;

impl Command for Metadata {
    fn name(&self) -> &str {
        "metadata"
    }

    fn usage(&self) -> &str {
        "Get the metadata for items in the stream."
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("metadata")
            .input_output_types(vec![(Type::Any, Type::Record(vec![]))])
            .allow_variants_without_examples(true)
            .optional(
                "expression",
                SyntaxShape::Any,
                "The expression you want metadata for.",
            )
            .switch(
                "data",
                "also add the data to the output record, under the key `data`",
                Some('d'),
            )
            .category(Category::Debug)
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let arg = call.positional_nth(0);
        let head = call.head;
        let include_data = call.has_flag(engine_state, stack, "data").unwrap_or(false);

        match arg {
            Some(Expression {
                expr: Expr::FullCellPath(full_cell_path),
                span,
                ..
            }) => {
                if full_cell_path.tail.is_empty() {
                    match &full_cell_path.head {
                        Expression {
                            expr: Expr::Var(var_id),
                            ..
                        } => {
                            let origin = stack.get_var_with_origin(*var_id, *span)?;

                            Ok(
                                build_metadata_record(Some(&origin), input, include_data, head)
                                    .into_pipeline_data(),
                            )
                        }
                        _ => {
                            let val: Value = call.req(engine_state, stack, 0)?;
                            Ok(
                                build_metadata_record(Some(&val), input, include_data, head)
                                    .into_pipeline_data(),
                            )
                        }
                    }
                } else {
                    let val: Value = call.req(engine_state, stack, 0)?;
                    Ok(
                        build_metadata_record(Some(&val), input, include_data, head)
                            .into_pipeline_data(),
                    )
                }
            }
            Some(_) => {
                let val: Value = call.req(engine_state, stack, 0)?;
                Ok(
                    build_metadata_record(Some(&val), input, include_data, head)
                        .into_pipeline_data(),
                )
            }
            None => {
                Ok(build_metadata_record(None, input, include_data, head).into_pipeline_data())
            }
        }
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Get the metadata of a variable",
                example: "let a = 42; metadata $a",
                result: None,
            },
            Example {
                description: "Get the metadata of the input",
                example: "ls | metadata",
                result: None,
            },
            Example {
                description: "Get the metadata of the input, along with the data",
                example: "ls | metadata --data",
                result: None,
            },
        ]
    }
}

fn build_metadata_record(arg: Option<&Value>, pipeline: PipelineData, include_data: bool, head: Span) -> Value {
    let mut record = Record::new();

    if let Some(span) = arg.map(Value::span) {
        record.push(
            "span",
            Value::record(
                record! {
                    "start" => Value::int(span.start as i64,span),
                    "end" => Value::int(span.end as i64, span),
                },
                head,
            ),
        );
    }

    if let Some(x) = pipeline.metadata().as_ref() {
        match x {
            PipelineMetadata {
                data_source: DataSource::Ls,
            } => record.push("source", Value::string("ls", head)),
            PipelineMetadata {
                data_source: DataSource::HtmlThemes,
            } => record.push("source", Value::string("into html --list", head)),
            PipelineMetadata {
                data_source: DataSource::FilePath(path),
            } => record.push(
                "source",
                Value::string(path.to_string_lossy().to_string(), head),
            ),
        }
    }

    if include_data {
        record.push("data", pipeline.into_value(head));
    }

    Value::record(record, head)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(Metadata {})
    }
}
