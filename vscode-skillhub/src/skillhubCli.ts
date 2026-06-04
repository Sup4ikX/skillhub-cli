import * as cp from "child_process";
import * as vscode from "vscode";

export interface Skill {
  name: string;
  version: string;
  description: string;
  author: string;
  tags: string[];
  agents: string[];
  installed?: boolean;
}

export interface SkillListResult {
  skills: Skill[];
  count: number;
}

export interface SkillInfoResult extends Skill {
  path?: string;
  license?: string;
  homepage?: string;
}

export interface StatsResult {
  installed: number;
  authors: number;
  agents_count: number;
  registry_size: number;
}

export function getCliPath(): string {
  const config = vscode.workspace.getConfiguration("skillhub");
  return config.get<string>("binaryPath", "skillhub");
}

function runCli(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    const cli = getCliPath();
    cp.execFile(cli, args, { timeout: 15000 }, (err, stdout, stderr) => {
      if (err) {
        reject(new Error(stderr || err.message));
        return;
      }
      resolve(stdout.trim());
    });
  });
}

export async function listInstalled(): Promise<SkillListResult> {
  const raw = await runCli(["list", "--json"]);
  return JSON.parse(raw);
}

export async function searchRegistry(query: string): Promise<SkillListResult> {
  const raw = await runCli(["search", query, "--json"]);
  return JSON.parse(raw);
}

export async function installSkill(name: string): Promise<string> {
  return await runCli(["install", name]);
}

export async function uninstallSkill(name: string): Promise<string> {
  return await runCli(["uninstall", name]);
}

export async function skillInfo(name: string): Promise<SkillInfoResult> {
  const raw = await runCli(["info", name, "--json"]);
  return JSON.parse(raw);
}

export async function getStats(): Promise<StatsResult> {
  const raw = await runCli(["stats", "--json"]);
  return JSON.parse(raw);
}
