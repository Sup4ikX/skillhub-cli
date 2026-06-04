import * as vscode from "vscode";
import * as cli from "./skillhubCli";

function skillTooltip(s: cli.Skill): string {
  return `${s.description}\nAuthor: ${s.author}\nVersion: ${s.version}`;
}

export class SkillTreeItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    skillOrNull: cli.Skill | null,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    public readonly type: "skill" | "installed" | "group" | "message"
  ) {
    super(label, collapsibleState);
    if (type === "skill") {
      this.contextValue = "skill";
      this.iconPath = new vscode.ThemeIcon("extensions");
      this.tooltip = `${skillTooltip(skillOrNull!)}\nAgents: ${skillOrNull!.agents.join(", ")}`;
      this.description = `v${skillOrNull!.version}`;
    } else if (type === "installed") {
      this.contextValue = "installed";
      this.iconPath = new vscode.ThemeIcon("check");
      this.tooltip = skillTooltip(skillOrNull!);
      this.description = `v${skillOrNull!.version} (installed)`;
    } else if (type === "group") {
      this.contextValue = "group";
      this.iconPath = new vscode.ThemeIcon("folder");
    } else {
      this.contextValue = "message";
      this.iconPath = new vscode.ThemeIcon("info");
    }
  }
}

function messageItem(text: string): SkillTreeItem {
  return new SkillTreeItem(text, null, vscode.TreeItemCollapsibleState.None, "message");
}

export class SkillTreeProvider implements vscode.TreeDataProvider<SkillTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<SkillTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private _mode: "installed" | "registry" = "installed";
  private _query = "";

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  setMode(mode: "installed" | "registry", query = ""): void {
    this._mode = mode;
    this._query = query;
    this.refresh();
  }

  getTreeItem(element: SkillTreeItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: SkillTreeItem): Promise<SkillTreeItem[]> {
    if (element) {
      return [];
    }
    try {
      if (this._mode === "installed") {
        return await this.getInstalledSkills();
      } else {
        return await this.getRegistrySkills();
      }
    } catch (e: any) {
      return [messageItem(`Error: ${e.message}`)];
    }
  }

  private async getInstalledSkills(): Promise<SkillTreeItem[]> {
    const result = await cli.listInstalled();
    if (!result.skills || result.skills.length === 0) {
      return [messageItem("No skills installed. Use search to find skills.")];
    }
    return result.skills.map(
      (s) =>
        new SkillTreeItem(s.name, s, vscode.TreeItemCollapsibleState.None, "installed")
    );
  }

  private async getRegistrySkills(): Promise<SkillTreeItem[]> {
    const query = this._query || "language";
    const result = await cli.searchRegistry(query);
    if (!result.skills || result.skills.length === 0) {
      return [messageItem("No skills found in registry.")];
    }
    return result.skills.map(
      (s) =>
        new SkillTreeItem(s.name, s, vscode.TreeItemCollapsibleState.None, "skill")
    );
  }
}
