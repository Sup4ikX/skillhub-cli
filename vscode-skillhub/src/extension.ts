import * as vscode from "vscode";
import { SkillTreeProvider } from "./treeView";
import * as cli from "./skillhubCli";
import { activateStatusBar, refreshStatusBar } from "./statusBar";

export function activate(context: vscode.ExtensionContext): void {
  const treeProvider = new SkillTreeProvider();
  const treeView = vscode.window.createTreeView("skillhubSkills", {
    treeDataProvider: treeProvider,
    showCollapseAll: true,
  });
  context.subscriptions.push(treeView);

  activateStatusBar(context);

  const refreshCmd = vscode.commands.registerCommand("skillhub.refresh", () => {
    treeProvider.setMode("installed");
    refreshStatusBar();
  });

  const searchCmd = vscode.commands.registerCommand("skillhub.search", async () => {
    const query = await vscode.window.showInputBox({
      prompt: "Search SkillHub registry",
      placeHolder: "e.g. python, react, docker",
    });
    if (query && query.trim()) {
      treeProvider.setMode("registry", query.trim());
    }
  });

  const installCmd = vscode.commands.registerCommand(
    "skillhub.install",
    async (item: any) => {
      const skillName = item?.skill?.name || item?.label;
      if (!skillName) {
        vscode.window.showErrorMessage("No skill selected");
        return;
      }
      try {
        const msg = await cli.installSkill(skillName);
        vscode.window.showInformationMessage(`Installed: ${skillName}`);
        treeProvider.refresh();
        refreshStatusBar();
      } catch (e: any) {
        vscode.window.showErrorMessage(`Install failed: ${e.message}`);
      }
    }
  );

  const uninstallCmd = vscode.commands.registerCommand(
    "skillhub.uninstall",
    async (item: any) => {
      const skillName = item?.skill?.name || item?.label;
      if (!skillName) {
        vscode.window.showErrorMessage("No skill selected");
        return;
      }
      const confirm = await vscode.window.showQuickPick(["Yes", "No"], {
        placeHolder: `Uninstall ${skillName}?`,
      });
      if (confirm !== "Yes") return;
      try {
        await cli.uninstallSkill(skillName);
        vscode.window.showInformationMessage(`Uninstalled: ${skillName}`);
        treeProvider.refresh();
        refreshStatusBar();
      } catch (e: any) {
        vscode.window.showErrorMessage(`Uninstall failed: ${e.message}`);
      }
    }
  );

  const infoCmd = vscode.commands.registerCommand(
    "skillhub.info",
    async (item: any) => {
      const skillName = item?.skill?.name || item?.label;
      if (!skillName) {
        vscode.window.showErrorMessage("No skill selected");
        return;
      }
      try {
        const info = await cli.skillInfo(skillName);
        const md = [
          `## ${info.name} v${info.version}`,
          `**Author:** ${info.author}`,
          `**Description:** ${info.description}`,
          `**Tags:** ${(info.tags || []).join(", ")}`,
          `**Agents:** ${(info.agents || []).join(", ")}`,
          info.license ? `**License:** ${info.license}` : "",
          info.homepage ? `**Homepage:** ${info.homepage}` : "",
          info.path ? `**Path:** \`${info.path}\`` : "",
        ]
          .filter(Boolean)
          .join("\n\n");
        const panel = vscode.window.createWebviewPanel(
          "skillhubInfo",
          `${info.name} — SkillHub`,
          vscode.ViewColumn.One,
          {}
        );
        panel.webview.html = `<!DOCTYPE html><html><body style="font-family: sans-serif; padding: 1em">${md.replace(
          /\n/g,
          "<br>"
        )}</body></html>`;
      } catch (e: any) {
        vscode.window.showErrorMessage(`Info failed: ${e.message}`);
      }
    }
  );

  const listInstalledCmd = vscode.commands.registerCommand(
    "skillhub.listInstalled",
    () => {
      treeProvider.setMode("installed");
    }
  );

  const openSettingsCmd = vscode.commands.registerCommand(
    "skillhub.openSettings",
    () => {
      vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "@ext:vscode-skillhub"
      );
    }
  );

  context.subscriptions.push(
    refreshCmd,
    searchCmd,
    installCmd,
    uninstallCmd,
    infoCmd,
    listInstalledCmd,
    openSettingsCmd
  );
}

export function deactivate(): void {
  // cleanup handled by context.subscriptions
}
