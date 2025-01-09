from pathlib import Path

import pytest
import pytest_lsp
from lsprotocol.types import (
    DidOpenTextDocumentParams,
    InitializeParams,
    PublishDiagnosticsParams,
    TextDocumentItem,
)
from pytest_lsp import ClientServerConfig, LanguageClient, client_capabilities


@pytest_lsp.fixture(
    config=ClientServerConfig(
        server_command=["../target/debug/codebook-lsp", "server"]
    ),
)
async def client(lsp_client: LanguageClient):
    # Setup
    params = InitializeParams(
        capabilities=client_capabilities("visual_studio_code"),
    )
    await lsp_client.initialize_session(params)
    yield
    # Teardown
    # Can't await here because of https://github.com/tower-lsp/tower-lsp/issues/12
    # LSP won't actually exit
    _ = lsp_client.shutdown_session()


# @pytest.mark.asyncio
# async def test_completions(client: LanguageClient):
#     """Ensure that the server implements completions correctly."""

#     client.text_document_did_open(
#         params=DidOpenTextDocumentParams(
#             position=Position(line=1, character=0),
#             text_document=TextDocumentIdentifier(uri="file:///path/to/file.txt"),
#         )
#     )

#     results = await client.
#     assert results is not None

#     if isinstance(results, CompletionList):
#         items = results.items
#     else:
#         items = results

#     labels = [item.label for item in items]
#     assert labels == ["hello", "world"]


@pytest.mark.asyncio
async def test_spellcheck_text(client: LanguageClient, tmp_path: Path):
    """Test that the server provides spelling diagnostics"""
    # Create a test file with intentional spelling mistakes
    test_file = tmp_path / "test.txt"
    test_file.write_text("This is an exampel of lenguage.")

    # Open the document
    uri = test_file.as_uri()
    client.text_document_did_open(
        params=DidOpenTextDocumentParams(
            text_document=TextDocumentItem(
                uri=uri,
                language_id="plaintext",
                version=1,
                text="This is an exampel of lenguage.",
            )
        )
    )

    # Wait for and get diagnostics
    diagnosticParams: PublishDiagnosticsParams = await client.wait_for_notification(
        "textDocument/publishDiagnostics"
    )
    diagnostics = diagnosticParams.diagnostics

    # Verify diagnostics
    assert len(diagnostics) == 2  # Should have two spelling errors

    # Check first diagnostic (exampel)
    assert diagnostics[0].range.start.line == 0
    assert diagnostics[0].range.start.character == 11
    assert diagnostics[0].range.end.line == 0
    assert diagnostics[0].range.end.character == 18
    assert "exampel" in diagnostics[0].message
    assert "example" in diagnostics[0].message

    # Check second diagnostic (lenguage)
    assert diagnostics[1].range.start.line == 0
    assert diagnostics[1].range.start.character == 22
    assert diagnostics[1].range.end.line == 0
    assert diagnostics[1].range.end.character == 30
    assert "lenguage" in diagnostics[1].message
    assert "language" in diagnostics[1].message
