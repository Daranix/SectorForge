import { invoke } from "@tauri-apps/api/core";
import type { DiskInfo } from "@/types/disk";

export async function listDisks(): Promise<DiskInfo[]> {
  return invoke<DiskInfo[]>("list_disks");
}

export async function formatSize(bytes: number): Promise<string> {
  return invoke<string>("format_size", { bytes });
}

export interface StartCloneArgs {
  sourcePath: string;
  outputPath: string;
  totalSize: number;
  verify: boolean;
}

export async function startClone(args: StartCloneArgs): Promise<void> {
  return invoke("start_clone", {
    sourcePath: args.sourcePath,
    outputPath: args.outputPath,
    totalSize: args.totalSize,
    verify: args.verify,
  });
}

export interface StartRestoreArgs {
  imagePath: string;
  targetPath: string;
}

export async function startRestore(args: StartRestoreArgs): Promise<void> {
  return invoke("start_restore", {
    imagePath: args.imagePath,
    targetPath: args.targetPath,
  });
}

export interface StartDiskToDiskArgs {
  sourcePath: string;
  targetPath: string;
  totalSize: number;
  verify: boolean;
}

export async function startDiskToDisk(args: StartDiskToDiskArgs): Promise<void> {
  return invoke("start_disk_to_disk", {
    sourcePath: args.sourcePath,
    targetPath: args.targetPath,
    totalSize: args.totalSize,
    verify: args.verify,
  });
}

export async function startVerify(imagePath: string): Promise<string> {
  return invoke<string>("start_verify", { imagePath });
}

export async function cancelOperation(): Promise<void> {
  return invoke("cancel_operation");
}

export async function isOperationRunning(): Promise<boolean> {
  return invoke<boolean>("is_operation_running");
}

export async function isElevated(): Promise<boolean> {
  return invoke<boolean>("is_elevated");
}

export async function restartAsAdmin(): Promise<void> {
  return invoke("restart_as_admin");
}
