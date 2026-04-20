import * as vscode from "vscode";
import { exec } from "node:child_process";

export function activate(context: vscode.ExtensionContext) {
  console.log("Gateorix extension activated");

  // Command: gateorix.init
  context.subscriptions.push(
    vscode.commands.registerCommand("gateorix.init", async () => {
      const name = await vscode.window.showInputBox({
        prompt: "Project name",
        placeHolder: "my-gateorix-app",
        validateInput: (value) => {
          if (!value || !/^[a-z0-9-]+$/.test(value)) {
            return "Use lowercase letters, numbers, and hyphens only";
          }
          return null;
        },
      });
      if (!name) return;

      const terminal = vscode.window.createTerminal("Gateorix");
      terminal.show();
      terminal.sendText(`gateorix init ${name}`);
    })
  );

  // Command: gateorix.dev
  context.subscriptions.push(
    vscode.commands.registerCommand("gateorix.dev", () => {
      const terminal = vscode.window.createTerminal("Gateorix Dev");
      terminal.show();
      terminal.sendText("gateorix dev");
    })
  );

  // Command: gateorix.build
  context.subscriptions.push(
    vscode.commands.registerCommand("gateorix.build", async () => {
      const mode = await vscode.window.showQuickPick(["debug", "release"], {
        placeHolder: "Build mode",
      });
      if (!mode) return;

      const terminal = vscode.window.createTerminal("Gateorix Build");
      terminal.show();
      terminal.sendText(mode === "release" ? "gateorix build --release" : "gateorix build");
    })
  );

  // Command: gateorix.doctor
  context.subscriptions.push(
    vscode.commands.registerCommand("gateorix.doctor", () => {
      const terminal = vscode.window.createTerminal("Gateorix Doctor");
      terminal.show();
      terminal.sendText("gateorix doctor");
    })
  );

  // Config file diagnostics
  const diagnostics = vscode.languages.createDiagnosticCollection("gateorix");
  context.subscriptions.push(diagnostics);

  const validateConfig = (document: vscode.TextDocument) => {
    if (!document.fileName.endsWith("gateorix.config.json")) return;

    const diags: vscode.Diagnostic[] = [];
    try {
      const config = JSON.parse(document.getText());
      if (!config.name) {
        diags.push(
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 1),
            "Missing required field: name",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
      if (!config.version) {
        diags.push(
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 1),
            "Missing required field: version",
            vscode.DiagnosticSeverity.Error
          )
        );
      }
    } catch {
      diags.push(
        new vscode.Diagnostic(
          new vscode.Range(0, 0, 0, 1),
          "Invalid JSON in gateorix.config.json",
          vscode.DiagnosticSeverity.Error
        )
      );
    }
    diagnostics.set(document.uri, diags);
  };

  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => validateConfig(e.document)),
    vscode.workspace.onDidOpenTextDocument(validateConfig)
  );

  // Status bar item
  const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.text = "$(zap) Gateorix";
  statusBar.command = "gateorix.dev";
  statusBar.tooltip = "Start Gateorix dev mode";
  context.subscriptions.push(statusBar);

  // Show status bar when in a Gateorix project
  vscode.workspace.findFiles("gateorix.config.json", null, 1).then((files) => {
    if (files.length > 0) {
      statusBar.show();
    }
  });
}

export function deactivate() {}
