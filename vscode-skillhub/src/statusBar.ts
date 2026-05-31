import * as vscode from "vscode";
import * as cli from "./skillhubCli";

let statusBarItem: vscode.StatusBarItem;

export function activateStatusBar(context: vscode.ExtensionContext): void {
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  statusBarItem.command = "skillhub.listInstalled";
  context.subscriptions.push(statusBarItem);
  updateStatusBar();
}

async function updateStatusBar(): Promise<void> {
  try {
    const stats = await cli.getStats();
    statusBarItem.text = `$(extensions) SkillHub: ${stats.installed} installed`;
    statusBarItem.tooltip = `${stats.registry_size} skills in registry | ${stats.agents_count} agents supported`;
    statusBarItem.show();
  } catch {
    statusBarItem.text = "$(extensions) SkillHub: not configured";
    statusBarItem.tooltip = "Run skillhub setup in terminal";
    statusBarItem.show();
  }
}

export function refreshStatusBar(): void {
  updateStatusBar();
}
