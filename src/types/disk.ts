export interface DiskInfo {
  DeviceID: string;
  Caption: string;
  Size: number;
  AvailableSpace: number;
  MediaType: string | null;
  Partitions: number;
  DriveLetters: string | null;
}

export interface ProgressPayload {
  bytes_processed: number;
  total_bytes: number;
  speed_bytes_per_sec: number;
  eta_seconds: number;
  percent: number;
}

export type OperationMode = "clone" | "restore" | "verify" | "disk2disk";
