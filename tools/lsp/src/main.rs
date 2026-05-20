// This implementation is based on the official one from `kdl-org/kdl-rs`, which
// in turn is based on the tower-lsp-boilerplate.

use std::path::PathBuf;

use dashmap::DashMap;
use packet_generator::kdl_parser::{
    ParserOpts, ParsingError, UnparsedKdl,
    schema::{JsonDefinition, RawDocument},
};
use ropey::Rope;
use tower_lsp::{Client, LanguageServer, LspService, Server, async_trait, jsonrpc, lsp_types::*};

fn char_to_position(char_idx: usize, rope: &Rope) -> Position {
    let line_idx = rope.char_to_line(char_idx);
    let line_char_idx = rope.line_to_char(line_idx);
    let column_idx = char_idx - line_char_idx;
    Position::new(line_idx as u32, column_idx as u32)
}

#[allow(dead_code)]
fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

fn position_to_offset(position: Position, rope: &Rope) -> Option<usize> {
    let line_char_offset = rope.try_line_to_char(position.line as usize).ok()?;
    let slice = rope.slice(0..line_char_offset + position.character as usize);
    Some(slice.len_bytes())
}

fn to_lsp_sev(sev: miette::Severity) -> DiagnosticSeverity {
    match sev {
        miette::Severity::Advice => DiagnosticSeverity::HINT,
        miette::Severity::Warning => DiagnosticSeverity::WARNING,
        miette::Severity::Error => DiagnosticSeverity::ERROR,
    }
}

#[derive()]
struct Backend {
    client: Client,
    files: DashMap<String, Rope>,
    semantic_map: DashMap<String, RawDocument>,
}

impl Backend {
    async fn on_change(&self, uri: Url, text: &str) {
        let rope = Rope::from_str(text);
        self.files.insert(uri.to_string(), rope.clone());
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        Ok(())
    }

    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::RegistrationOptions(
                    DiagnosticRegistrationOptions {
                        text_document_registration_options: TextDocumentRegistrationOptions {
                            document_selector: Some(vec![DocumentFilter {
                                language: Some("packet-generator-kdl".into()),
                                scheme: Some("file".into()),
                                pattern: None,
                            }]),
                        },
                        ..Default::default()
                    },
                )),
                // TODO(anri):
                // Completions need heavy parser support.
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                // implementation_provider: (),
                // references_provider: (),
                // document_highlight_provider: (),
                // document_symbol_provider: (),
                // workspace_symbol_provider: (),
                // color_provider: (),
                // declaration_provider: (),
                // execute_command_provider: (),
                // inlay_hint_provider: (),
                // type_definition_provider: (),
                // hover_provider: (),
                // definition_provider: (),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: String::from("packet-generator-lsp"),
                version: Some(String::from("0.1.0")),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.files.remove(&params.text_document.uri.to_string());
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(params.text_document.uri, &params.content_changes[0].text)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text.as_ref() {
            self.on_change(params.text_document.uri, text).await;
        }
    }

    // TODO(anri):
    // Completions need heavy parser support.
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, jsonrpc::Error> {
        let uri = params.text_document_position.text_document.uri;
        let uri_str = uri.to_string();
        let path = uri.path();

        let (source, document) = match (self.files.get(&uri_str), self.semantic_map.get(&uri_str)) {
            (Some(source), Some(document)) => (source, document),
            _ => return Ok(None),
        };

        let Some(position) = position_to_offset(params.text_document_position.position, &source)
        else {
            return Ok(None);
        };

        async fn search_start(document: &RawDocument, path: &str, char_off: usize) -> bool {
            let maybe_json = document
                .json_definitions
                .iter()
                .find(|def: &&JsonDefinition| {
                    char_off >= def.span.offset()
                        && char_off <= (def.span.offset() + def.span.len())
                        && def.source_info.name == path
                });

            maybe_json.is_some()
        }

        let mut completions = vec![];

        if search_start(&document, path, position).await {
            completions.push(CompletionItem {
                label: String::from("doc"),
                // label_details: todo!(),
                kind: Some(CompletionItemKind::FIELD),
                // detail: todo!(),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from("A documentation child"),
                })),
                ..Default::default()
            });

            completions.push(CompletionItem {
                label: String::from("key"),
                // label_details: todo!(),
                kind: Some(CompletionItemKind::FIELD),
                // detail: todo!(),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from("The key for this JSON"),
                })),
                ..Default::default()
            });

            completions.push(CompletionItem {
                label: String::from("field"),
                // label_details: todo!(),
                kind: Some(CompletionItemKind::FIELD),
                // detail: todo!(),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from("A JSON field"),
                })),
                ..Default::default()
            });

            Ok(Some(CompletionResponse::Array(completions)))
        } else {
            Ok(None)
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult, jsonrpc::Error> {
        let default_return = DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport::default()),
        );

        let uri = params.text_document.uri.to_string();

        let Some(file) = self.files.get(&uri) else {
            return Ok(default_return);
        };

        let path = params.text_document.uri.path();

        let content = file.to_string();
        let path = PathBuf::from(path);

        let unparsed_kdl = UnparsedKdl::new_owned(content, path);

        let res =
            packet_generator::kdl_parser::raw_parse_kdl(&[unparsed_kdl], &ParserOpts::default());

        match res {
            Ok((document, warnings)) => {
                self.semantic_map.insert(uri, document);

                let new_diagnostics: Vec<_> = warnings
                    .iter()
                    .map(|diag| {
                        Diagnostic::new(
                            Range::new(
                                char_to_position(diag.span.offset(), &file),
                                char_to_position(diag.span.offset() + diag.span.len(), &file),
                            ),
                            Some(to_lsp_sev(diag.severity)),
                            None,
                            None,
                            diag.message.clone(),
                            None,
                            None,
                        )
                    })
                    .collect();

                if new_diagnostics.is_empty() {
                    Ok(default_return)
                } else {
                    Ok(DocumentDiagnosticReportResult::Report(
                        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items: new_diagnostics,
                            },
                        }),
                    ))
                }
            }

            Err(errors) => match errors {
                ParsingError::KdlError(kdl_error) => {
                    let diags = kdl_error
                        .diagnostics
                        .into_iter()
                        .map(|diag| {
                            Diagnostic::new(
                                Range::new(
                                    char_to_position(diag.span.offset(), &file),
                                    char_to_position(diag.span.offset() + diag.span.len(), &file),
                                ),
                                Some(to_lsp_sev(diag.severity)),
                                None,
                                None,
                                diag.to_string(),
                                None,
                                None,
                            )
                        })
                        .collect();

                    Ok(DocumentDiagnosticReportResult::Report(
                        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items: diags,
                            },
                        }),
                    ))
                }

                ParsingError::Diagnostics {
                    source_info: _,
                    diagnostics,
                } => {
                    let new_diagnostics = diagnostics
                        .into_iter()
                        .map(|diag| {
                            Diagnostic::new(
                                Range::new(
                                    char_to_position(diag.span.offset(), &file),
                                    char_to_position(diag.span.offset() + diag.span.len(), &file),
                                ),
                                Some(to_lsp_sev(diag.severity)),
                                None,
                                None,
                                diag.message,
                                None,
                                None,
                            )
                        })
                        .collect();

                    Ok(DocumentDiagnosticReportResult::Report(
                        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items: new_diagnostics,
                            },
                        }),
                    ))
                }

                _ => Ok(default_return),
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        files: DashMap::new(),
        semantic_map: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
