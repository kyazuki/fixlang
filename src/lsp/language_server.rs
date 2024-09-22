use crate::ast::expr::ExprNode;
use crate::ast::program::Program;
use crate::constants::INSTANCIATED_NAME_SEPARATOR;
use crate::FullName;
use crate::{
    constants::LSP_LOG_FILE_PATH,
    error::{any_to_string, Error, Errors},
    project_file::ProjectFile,
    runner::build_file,
    Configuration, Span,
};
use difference::diff;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    CompletionParams, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Documentation, GotoDefinitionParams, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, MarkupContent,
    PublishDiagnosticsParams, SaveOptions, ServerCapabilities, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, WorkDoneProgressOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

pub const WRITE_LOG: bool = true;

#[derive(Deserialize, Serialize)]
pub struct JSONRPCMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl JSONRPCMessage {
    pub fn new(
        id: Option<u32>,
        method: Option<String>,
        params: Option<Value>,
        result: Option<Value>,
        error: Option<Value>,
    ) -> Self {
        JSONRPCMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
            result,
            error,
        }
    }
}

// Requests sent to diagnostic thread.
enum DiagnosticsMessage {
    // Started the diagnostics thread.
    Start,
    // A file is saved.
    OnSaveFile,
    // Stop the diagnostics thread.
    Stop,
}

// The result of diagnostics.
pub struct DiagnosticsResult {
    pub prgoram: Program,
}

// Launch the language server
pub fn launch_language_server() {
    let mut stdin = std::io::stdin();

    // Prepare the log file.
    let log_file = open_log_file();
    write_log(log_file.clone(), "Language server started.\n");

    // Prepare a channel to send requests to the diagnostics thread.
    let (diag_req_send, diag_req_recv) = mpsc::channel::<DiagnosticsMessage>();
    let mut diag_req_recv = Some(diag_req_recv);

    // Prepare a channel to response from the diagnostics thread.
    let (diag_res_send, diag_res_recv) = mpsc::channel::<DiagnosticsResult>();

    // The last diagnostics result.
    let mut last_diag: Option<DiagnosticsResult> = None;

    // Maps to get file contents from Uris.
    let mut uri_to_latest_content: HashMap<lsp_types::Uri, String> =
        std::collections::HashMap::new();

    loop {
        // If new diagnostics are available, send store it to `last_diag`.
        if let Ok(res) = diag_res_recv.try_recv() {
            last_diag = Some(res);
        }

        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            let mut msg = "Failed to read a line: \n".to_string();
            msg.push_str(&format!("{:?}\n", res.unwrap_err()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            let mut msg = "Expected `Content-Length:`. The line is: \n".to_string();
            msg.push_str(&format!("{:?}\n", content_length));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: Result<usize, _> = content_length
            .split_off("Content-Length:".len())
            .trim()
            .parse();
        if content_length.is_err() {
            let mut msg = "Failed to parse the content length: \n".to_string();
            msg.push_str(&format!("{:?}\n", content_length.err().unwrap()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        let content_length = content_length.unwrap();

        // Read stdin upto an empty line.
        loop {
            let mut line = String::new();
            let res = stdin.read_line(&mut line);
            if res.is_err() {
                let e = res.unwrap_err();
                let mut msg = "Failed to read a line: \n".to_string();
                msg.push_str(&format!("{:?}\n", e));
                write_log(log_file.clone(), msg.as_str());
                continue;
            }
            if line.trim().is_empty() {
                break;
            }
        }

        // Read the content of the message.
        let mut message = vec![0; content_length];
        let res = stdin.read_exact(&mut message);
        if res.is_err() {
            let mut msg = "Failed to read the message: \n".to_string();
            msg.push_str(&format!("{:?}\n", res.unwrap_err()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log(
                log_file.clone(),
                "Failed to parse the message as utf-8 string: \n",
            );
            write_log(log_file.clone(), &format!("{:?}\n", message.unwrap_err()));
            continue;
        }
        let message = message.unwrap();

        // Parse the message as JSONRPCMessage.
        let message: Result<JSONRPCMessage, _> = serde_json::from_str(&message);
        if message.is_err() {
            write_log(
                log_file.clone(),
                "Failed to parse the message as JSONRPCMessage: \n",
            );
            write_log(log_file.clone(), &format!("{:?}\n", message.err().unwrap()));
            continue;
        }
        let message = message.unwrap();

        // Depending on the method, handle the message.
        if let Some(method) = message.method.as_ref() {
            if method == "initialize" {
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<InitializeParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_initialize(id.unwrap(), &params.unwrap(), log_file.clone());
            } else if method == "initialized" {
                let params: Option<InitializedParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                if diag_req_recv.is_none() {
                    let msg = "\"initialized\" method is sent twice.\n".to_string();
                    write_log(log_file.clone(), msg.as_str());
                    continue;
                }
                handle_initialized(
                    &params.unwrap(),
                    diag_req_send.clone(),
                    diag_req_recv.take().unwrap(),
                    diag_res_send.clone(),
                    log_file.clone(),
                );
            } else if method == "shutdown" {
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                handle_shutdown(id.unwrap(), diag_req_send.clone(), log_file.clone());
            } else if method == "exit" {
                write_log(log_file.clone(), "Exiting the language server.\n");
                break;
            } else if method == "textDocument/didOpen" {
                let params: Option<DidOpenTextDocumentParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_open(
                    &params.unwrap(),
                    &mut uri_to_latest_content,
                    log_file.clone(),
                );
            } else if method == "textDocument/didChange" {
                let params: Option<DidChangeTextDocumentParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_change(
                    &params.unwrap(),
                    &mut uri_to_latest_content,
                    log_file.clone(),
                );
            } else if method == "textDocument/didSave" {
                let params: Option<DidSaveTextDocumentParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_save(
                    diag_req_send.clone(),
                    &params.unwrap(),
                    &mut uri_to_latest_content,
                    log_file.clone(),
                );
            } else if method == "textDocument/completion" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().prgoram;
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<CompletionParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_completion(id.unwrap(), &params.unwrap(), program, log_file.clone());
            } else if method == "completionItem/resolve" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().prgoram;
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<CompletionItem> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_completion_resolve_document(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    log_file.clone(),
                );
            } else if method == "textDocument/hover" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().prgoram;
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<HoverParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_hover(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                    log_file.clone(),
                );
            } else if method == "textDocument/definition" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().prgoram;
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<GotoDefinitionParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_goto_definition(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                    log_file.clone(),
                );
            }
        }
    }
}

fn parase_params<T: DeserializeOwned>(params: Value, log_file: Arc<Mutex<File>>) -> Option<T> {
    let params: Result<T, _> = serde_json::from_value(params);
    if params.is_err() {
        let mut msg = "Failed to parse the params: \n".to_string();
        msg.push_str(&format!("{:?}\n", params.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return None;
    }
    params.ok()
}

fn parse_id(message: &JSONRPCMessage, method: &str, log_file: Arc<Mutex<File>>) -> Option<u32> {
    if message.id.is_none() {
        write_log(
            log_file,
            format!(
                "Failed to get \"id\" from the message for method \"{}\".\n",
                method
            )
            .as_str(),
        );
        return None;
    }
    message.id
}

#[allow(dead_code)]
fn send_request<T: Serialize>(id: u32, method: String, params: Option<T>) {
    let msg = JSONRPCMessage::new(
        Some(id),
        Some(method),
        params.map(|params| serde_json::to_value(params).unwrap()),
        None,
        None,
    );
    send_message(&msg);
}

fn send_response<T: Serialize, E: Serialize>(id: u32, result: Result<T, E>) {
    let (res, err) = match result {
        Ok(res) => (Some(res), None),
        Err(err) => (None, Some(err)),
    };
    let msg = JSONRPCMessage::new(
        Some(id),
        None,
        None,
        res.map(|res| serde_json::to_value(res).unwrap()),
        err.map(|err| serde_json::to_value(err).unwrap()),
    );
    send_message(&msg);
}

fn send_notification<T: Serialize>(method: String, params: Option<T>) {
    let msg = JSONRPCMessage::new(
        None,
        Some(method),
        params.map(|params| serde_json::to_value(params).unwrap()),
        None,
        None,
    );
    send_message(&msg);
}

fn send_message(msg: &JSONRPCMessage) {
    let msg = serde_json::to_string(msg).unwrap();
    let content_length = msg.len();
    print!("Content-Length: {}\r\n\r\n{}", content_length, msg);
    std::io::stdout()
        .flush()
        .expect("Failed to flush the stdout.");
}

fn open_log_file() -> Arc<Mutex<File>> {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LSP_LOG_FILE_PATH)
        .parent()
        .expect("Failed to get parent directory of LSP_LOG_FILE_PATH.");

    // Create directories to the parent directory.
    std::fs::create_dir_all(parent_dir)
        .expect("Failed to create directories to the parent directory.");

    // Create and open the log file.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LSP_LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LSP_LOG_FILE_PATH).as_str());

    // Wrap it into a mutex.
    Arc::new(Mutex::new(file))
}

fn write_log(file: Arc<Mutex<File>>, message: &str) {
    let mut file = file.lock().expect("Failed to lock the log file.");
    if WRITE_LOG {
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

// Handle "initialize" method.
fn handle_initialize(id: u32, _params: &InitializeParams, _log_file: Arc<Mutex<File>>) {
    // Return server capabilities.
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: None,
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(true),
                    })),
                },
            )),
            notebook_document_sync: None,
            selection_range_provider: None,
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![
                    " ".to_string(),
                    ".".to_string(),
                    "(".to_string(),
                    ":".to_string(),
                ]),
                all_commit_characters: None,
                resolve_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: None,
            }),
            signature_help_provider: None,
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: None,
            document_highlight_provider: None,
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: None,
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: None,
            document_link_provider: None,
            color_provider: None,
            folding_range_provider: None,
            declaration_provider: None,
            execute_command_provider: None,
            workspace: None,
            call_hierarchy_provider: None,
            semantic_tokens_provider: None,
            moniker_provider: None,
            linked_editing_range_provider: None,
            inline_value_provider: None,
            inlay_hint_provider: None,
            diagnostic_provider: None,
            experimental: None,
        },
        server_info: None,
    };
    send_response(id, Ok::<_, ()>(result))
}

// Handle "initialized" method.
fn handle_initialized(
    _params: &InitializedParams,
    diag_req_send: Sender<DiagnosticsMessage>,
    diag_req_recv: Receiver<DiagnosticsMessage>,
    diag_res_send: Sender<DiagnosticsResult>,
    log_file: Arc<Mutex<File>>,
) {
    // Launch the diagnostics thread.
    let log_file_cloned = log_file.clone();
    std::thread::spawn(|| {
        let res = std::panic::catch_unwind(|| {
            diagnostics_thread(diag_req_recv, diag_res_send, log_file_cloned.clone());
        });
        if res.is_err() {
            // If a panic occurs in the diagnostics thread,
            send_diagnostics_error_message(
                "Diagnostics stopped. This may be a bug of \"fix\" command. I would be happy if you report how to reproduce this! Repo: https://github.com/tttmmmyyyy/fixlang".to_string(),
                log_file_cloned.clone(),
            );
            let mut msg = "Panic occurred in the diagnostics thread: \n".to_string();
            msg.push_str(&format!("{}\n", any_to_string(res.err().as_ref().unwrap())));
            write_log(log_file_cloned, msg.as_str());
        }
    });

    // Send `Start` message to the diagnostics thread.
    if let Err(e) = diag_req_send.send(DiagnosticsMessage::Start) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(log_file.clone(), msg.as_str());
    }
}

// Handle "shutdown" method.
fn handle_shutdown(id: u32, diag_send: Sender<DiagnosticsMessage>, _log_file: Arc<Mutex<File>>) {
    // Shutdown the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::Stop) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(_log_file.clone(), msg.as_str());
    }

    // Respond to the client.
    let param = Ok::<_, ()>(serde_json::to_value(None::<()>).unwrap());
    send_response(id, param);
}

// Handle "textDocument/didOpen" method.
fn handle_textdocument_did_open(
    params: &DidOpenTextDocumentParams,
    uri_to_latest_content: &mut HashMap<lsp_types::Uri, String>,
    _log_file: Arc<Mutex<File>>,
) {
    // Store the content of the file into the maps.
    uri_to_latest_content.insert(
        params.text_document.uri.clone(),
        params.text_document.text.clone(),
    );
}

// Handle "textDocument/didChange" method.
fn handle_textdocument_did_change(
    params: &DidChangeTextDocumentParams,
    uri_to_latest_content: &mut HashMap<lsp_types::Uri, String>,
    _log_file: Arc<Mutex<File>>,
) {
    // Store the content of the file into `uri_to_content`.
    if let Some(change) = params.content_changes.last() {
        uri_to_latest_content.insert(params.text_document.uri.clone(), change.text.clone());
    }
}

// Handle "textDocument/didSave" method.
fn handle_textdocument_did_save(
    diag_send: Sender<DiagnosticsMessage>,
    params: &DidSaveTextDocumentParams,
    uri_to_latest_content: &mut HashMap<lsp_types::Uri, String>,
    log_file: Arc<Mutex<File>>,
) {
    // Store the content of the file into maps.
    if let Some(text) = &params.text {
        uri_to_latest_content.insert(params.text_document.uri.clone(), text.clone());
    } else {
        let msg = "No text data in \"textDocument/didSave\" notification.".to_string();
        write_log(log_file.clone(), msg.as_str());
    }

    // Send a message to the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::OnSaveFile) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(log_file.clone(), msg.as_str());
    }
}

// Handle "textDocument/completion" method.
fn handle_completion(
    id: u32,
    _params: &CompletionParams,
    program: &Program,
    _log_file: Arc<Mutex<File>>,
) {
    let mut items = vec![];
    for (full_name, gv) in &program.global_values {
        let name = full_name.name.clone();
        // Skip compiler-defined values.
        if name.starts_with(INSTANCIATED_NAME_SEPARATOR) {
            continue;
        }
        let in_namespace = " in ".to_string() + &full_name.namespace.to_string();
        let scheme = gv.scm.to_string();

        items.push(CompletionItem {
            label: name,
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(in_namespace),
                description: None,
            }),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(scheme),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: Some(serde_json::to_value(full_name.to_string()).unwrap()), // Full name of the global value.
            tags: None,
        });
    }
    send_response(id, Ok::<_, ()>(items));
}

// Handle "textDocument/completion" method.
// Add documentation to the completion item.
fn handle_completion_resolve_document(
    id: u32,
    params: &CompletionItem,
    program: &Program,
    log_file: Arc<Mutex<File>>,
) {
    // Extract the full name of the global value for which completion is requested from the params.
    if params.data.is_none() {
        let msg = "Failed to get the data from the params.".to_string();
        write_log(log_file.clone(), msg.as_str());
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }
    let full_name_str = &params.data.as_ref().unwrap().as_str().unwrap();
    let full_name = FullName::parse(full_name_str);
    if full_name.is_none() {
        let msg = format!("Failed to parse the full name `{}`.", full_name_str);
        write_log(log_file.clone(), msg.as_str());
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }
    let full_name = full_name.unwrap();

    // Find the global value requested for completion.
    let gv = program.global_values.get(&full_name);
    if gv.is_none() {
        let msg = format!("No value named `{}` is found.", full_name_str);
        write_log(log_file.clone(), msg.as_str());
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }
    let gv = gv.unwrap();

    // Get the documentation.
    let docs = gv.get_document();

    // Set the documentation into the given completion item.
    let docs = docs.map(|doc_str| {
        Documentation::MarkupContent(MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: doc_str,
        })
    });
    let mut item = params.clone();
    item.documentation = docs;

    // Send the completion item.
    send_response(id, Ok::<_, ()>(item));
}

fn get_file_content_at_previous_diagnostics(
    program: &Program,
    path: &Path,
) -> Result<String, String> {
    for (_, src) in &program.module_to_files {
        if src.file_path == path {
            let content = src.string();
            if let Err(_e) = content {
                let msg = format!(
                    "Failed to get the content of the file: \"{}\"",
                    src.file_path.to_string_lossy().to_string()
                );
                return Err(msg);
            }
            return Ok(content.ok().unwrap());
        }
    }
    let msg = format!(
        "No saved content for the file: \"{}\"\n",
        path.to_string_lossy().to_string()
    );
    return Err(msg);
}

fn calculate_corresponding_line(content0: &str, content1: &str, line0: u32) -> Option<u32> {
    let (_, diffs) = diff(content0, content1, "\n");
    let mut line_cnt_0 = -1;
    let mut line_cnt_1 = -1;
    for diff in diffs {
        match diff {
            difference::Difference::Same(s) => {
                let lines = s.split("\n").count();
                for _ in 0..lines {
                    line_cnt_0 += 1;
                    line_cnt_1 += 1;
                    if line_cnt_0 == line0 as i32 {
                        return Some(line_cnt_1 as u32);
                    }
                }
            }
            difference::Difference::Add(s) => {
                line_cnt_1 += s.split("\n").count() as i32;
            }
            difference::Difference::Rem(s) => {
                line_cnt_0 += s.split("\n").count() as i32;
            }
        }
    }
    None
}

fn get_node_at(
    text_position: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &HashMap<lsp_types::Uri, String>,
    log_file: Arc<Mutex<File>>,
) -> Option<Arc<ExprNode>> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log(log_file.clone(), msg.as_str());
        let msg = format!("{:?}", uri_to_content);
        write_log(log_file.clone(), msg.as_str());
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();

    // Get the path of the file.
    let path = PathBuf::from(uri.path().to_string());

    // Get the file content at the time of the last successful diagnostics.
    let saved_content = get_file_content_at_previous_diagnostics(program, &path);
    if let Err(e) = saved_content {
        write_log(log_file.clone(), &e);
        return None;
    }
    let saved_content = saved_content.ok().unwrap();

    // Get the position of the cursor in `saved_content`.
    let pos_in_latest = text_position.position;
    let line_in_saved =
        calculate_corresponding_line(latest_content, &saved_content, pos_in_latest.line);
    if line_in_saved.is_none() {
        return None;
    }
    let pos_in_saved = lsp_types::Position {
        line: line_in_saved.unwrap(),
        character: pos_in_latest.character,
    };

    // Get the node at the position.
    let pos = position_to_bytes(&saved_content, pos_in_saved);
    program.find_node_at(&path, pos)
}

// Handle "textDocument/definition" method.
fn handle_goto_definition(
    id: u32,
    params: &GotoDefinitionParams,
    program: &Program,
    uri_to_content: &HashMap<lsp_types::Uri, String>,
    log_file: Arc<Mutex<File>>,
) {
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
        log_file.clone(),
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let node = node.unwrap();

    // if the node is not a variable, nothing to do.
    if !node.is_var() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }

    // If the variable is local, do nothing.
    let full_name = &node.get_var().name;
    if full_name.is_local() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }

    // Find the definition of the global value.
    let gv = program.global_values.get(full_name);
    if gv.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let gv = gv.unwrap();
    let def_src = &gv.def_src;
    if def_src.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let def_src = def_src.as_ref().unwrap();

    // Create response value.
    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}\n", cdir.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return;
    }
    let cdir = cdir.unwrap();
    let uri = path_to_uri(&cdir.join(&def_src.input.file_path));
    if let Err(e) = uri {
        send_response(id, Err::<(), String>(e));
        return;
    }
    let uri = uri.ok().unwrap();
    let location = lsp_types::Location {
        uri,
        range: span_to_range(&def_src),
    };
    send_response(id, Ok::<_, ()>(location));
}

// Handle "textDocument/hover" method.
fn handle_hover(
    id: u32,
    params: &HoverParams,
    program: &Program,
    uri_to_content: &HashMap<lsp_types::Uri, String>,
    log_file: Arc<Mutex<File>>,
) {
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
        log_file,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let node = node.unwrap();

    // if the node is not a variable, nothing to do.
    if !node.is_var() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }

    // Get informations of the variable which are needed to show in the hover.
    let full_name = &node.get_var().name;
    let ty = &node.ty;

    // Create a hover message.
    let mut docs = String::new();
    if full_name.is_local() {
        // In case the variable is local
        if let Some(ty) = ty.as_ref() {
            docs += &format!("`{} : {}`", full_name.to_string(), ty.to_string_normalize());
        } else {
            docs += &format!("`{}`", full_name.to_string());
        }
    } else {
        // In case the variable is global, show the documentation of the global value.
        docs += &format!("`{}`", full_name.to_string());
        let mut scm_string = String::new();
        if let Some(gv) = program.global_values.get(full_name) {
            scm_string = gv.scm.to_string();
            docs += &format!("\n- Defined as `{}`", scm_string);
        }
        if let Some(ty) = ty.as_ref() {
            let ty_string = ty.to_string_normalize();
            if scm_string != ty_string {
                docs += &format!("\n- Used as `{}`", ty_string);
            }
        }
        if let Some(gv) = program.global_values.get(full_name) {
            if let Some(document) = gv.get_document() {
                docs += &format!("\n\n{}", document);
            }
        }
    };
    let content = MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: docs,
    };
    let hover = lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(content),
        range: None,
    };
    send_response(id, Ok::<_, ()>(hover))
}

// Convert a `lsp_types::Position` into a bytes position in a string.
fn position_to_bytes(string: &str, position: lsp_types::Position) -> usize {
    let mut bytes = 0;
    let mut line = 0;
    let mut pos = 0;
    for c in string.chars() {
        bytes += c.len_utf8();
        pos += 1;
        if c == '\n' {
            line += 1;
            pos = 0;
        }
        if line == position.line && pos == position.character as usize {
            break;
        }
    }
    bytes
}

// The entry point of the diagnostics thread.
fn diagnostics_thread(
    req_recv: Receiver<DiagnosticsMessage>,
    res_send: Sender<DiagnosticsResult>,
    log_file: Arc<Mutex<File>>,
) {
    let mut prev_err_paths = HashSet::new();

    loop {
        // Wait for a message.
        let msg = req_recv.recv();
        if msg.is_err() {
            // If the sender is dropped, stop the diagnostics thread.
            break;
        }

        // Run diagnostics.
        let res = match msg.unwrap() {
            DiagnosticsMessage::Stop => {
                // Stop the diagnostics thread.
                break;
            }
            DiagnosticsMessage::OnSaveFile => run_diagnostics(log_file.clone()),
            DiagnosticsMessage::Start => run_diagnostics(log_file.clone()),
        };

        // Send the result to the main thread and language clinent.
        let errs = match res {
            Ok(res) => {
                res_send.send(res).unwrap();
                Errors::empty()
            }
            Err(errs) => errs,
        };
        prev_err_paths = send_diagnostics_notification(
            errs,
            std::mem::replace(&mut prev_err_paths, HashSet::new()),
            log_file.clone(),
        );
    }
}

// Convert a `Span` into a `Range`.
fn span_to_range(span: &Span) -> lsp_types::Range {
    fn pair_to_zero_indexed((x, y): (usize, usize)) -> (usize, usize) {
        (x - 1, y - 1)
    }

    let (start_line, start_column) = pair_to_zero_indexed(span.start_line_col());
    let (end_line, end_column) = pair_to_zero_indexed(span.end_line_col());
    lsp_types::Range {
        start: lsp_types::Position {
            line: start_line as u32,
            character: start_column as u32,
        },
        end: lsp_types::Position {
            line: end_line as u32,
            character: end_column as u32,
        },
    }
}

// Send the diagnostics notification to the client.
// Return the paths of the files that have errors.
// - `prev_err_paths`: The paths of the files that have errors in the previous diagnostics. This is used to clear the diagnostics for the files that have no errors.
fn send_diagnostics_notification(
    errs: Errors,
    mut prev_err_paths: HashSet<PathBuf>,
    log_file: Arc<Mutex<File>>,
) -> HashSet<PathBuf> {
    let mut err_paths = HashSet::new();

    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}\n", cdir.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return err_paths;
    }
    let cdir = cdir.unwrap();

    // Send the diagnostics notification for each file that has errors.
    for (path, errs) in errs.organize_by_path() {
        err_paths.insert(path.clone());
        prev_err_paths.remove(&path);

        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(path));
        if uri.is_err() {
            write_log(
                log_file.clone(),
                &format!("Failed to convert path to uri: {:?}\n", uri.unwrap_err()),
            );
            continue;
        }
        let uri = uri.unwrap();

        // Send the diagnostics notification for each file.
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics: errs
                .iter()
                .map(|err| error_to_diagnostics(err, &cdir, log_file.clone()))
                .collect(),
            version: None,
        };
        send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
    }

    // Clear the diagnostics for the files that have no errors.
    for path in prev_err_paths {
        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(path));
        if uri.is_err() {
            write_log(log_file.clone(), &(uri.unwrap_err() + "\n"));
            continue;
        }
        let uri = uri.unwrap();

        // Send the empty diagnostics notification for each file.
        let params = lsp_types::PublishDiagnosticsParams {
            uri,
            diagnostics: vec![],
            version: None,
        };
        send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
    }

    err_paths
}

// Send the diagnostics notification to the client which informs that an error occurred.
fn send_diagnostics_error_message(msg: String, log_file: Arc<Mutex<File>>) {
    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}\n", cdir.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return;
    }
    let cdir = cdir.unwrap();
    // Convert path to uri.
    let cdir_uri = path_to_uri(&cdir);
    if cdir_uri.is_err() {
        write_log(
            log_file.clone(),
            &format!(
                "Failed to convert path to uri: {:?}\n",
                cdir_uri.unwrap_err()
            ),
        );
        return;
    }
    let cdir_uri = cdir_uri.unwrap();

    // Send the diagnostics notification for each file.
    let params = lsp_types::PublishDiagnosticsParams {
        uri: cdir_uri,
        diagnostics: vec![lsp_types::Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: None,
            message: msg,
            tags: None,
            related_information: None,
            data: None,
        }],
        version: None,
    };
    send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
}

// Convert an `Error` into a diagnostic message.
fn error_to_diagnostics(
    err: &Error,
    cdir: &PathBuf,
    log_file: Arc<Mutex<File>>,
) -> lsp_types::Diagnostic {
    // Show error at the first span in `err`.
    let range = err
        .srcs
        .first()
        .map(|span| span_to_range(span))
        .unwrap_or_default();

    // Other spans are shown in related informations.
    let mut related_information = vec![];
    for span in err.srcs.iter().skip(1) {
        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(&span.input.file_path));
        if uri.is_err() {
            write_log(
                log_file.clone(),
                &format!("Failed to convert path to uri: {:?}\n", uri.unwrap_err()),
            );
            continue;
        }
        let uri = uri.unwrap();

        // Create related informations.
        let related = lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri,
                range: span_to_range(span),
            },
            message: "see also here".to_string(),
        };
        related_information.push(related);
    }
    let related_information = if related_information.is_empty() {
        None
    } else {
        Some(related_information)
    };

    lsp_types::Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: None,
        message: err.msg.clone(),
        tags: None,
        related_information,
        data: None,
    }
}

fn path_to_uri(path: &PathBuf) -> Result<lsp_types::Uri, String> {
    let path = path.to_str();
    if path.is_none() {
        return Err(format!("Failed to convert a path into string: {:?}", path));
    }
    let path = "file://".to_string() + path.unwrap();
    let uri = lsp_types::Uri::from_str(&path);
    if uri.is_err() {
        return Err(format!("Failed to convert a path into Uri: {:?}", path));
    }
    Ok(uri.unwrap())
}

fn run_diagnostics(_log_file: Arc<Mutex<File>>) -> Result<DiagnosticsResult, Errors> {
    // TODO: maybe we should check if the file has been changed actually after previous diagnostics?

    // Read the project file.
    let proj_file = ProjectFile::read_root_file()?;

    // Create the configuration.
    let mut config = Configuration::language_server()?;

    // Set up the configuration by the project file and the lock file.
    proj_file.set_config(&mut config, false)?;
    proj_file.open_lock_file()?.set_config(&mut config)?;

    // Build the file and get the errors.
    let program = build_file(&mut config)?.program.unwrap();

    Ok(DiagnosticsResult { prgoram: program })
}
