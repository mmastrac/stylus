// Type definitions for Stylus status JSON

export interface ServerConfig {
  port: number;
  listen_addr: string;
  static: string;
}

export interface MonitorConfig {
  dir: string;
}

export interface CSSConfig {
  metadata: Record<string, any>;
  rules: any[];
}

export interface StackRow {
  id: string;
  size: string;
  layout: string;
}

export interface Stack {
  title: string;
  rows: StackRow[];
}

export interface RowColumn {
  type: string;
  width: number;
  url?: string;
  config?: string;
  inject?: boolean;
  stacks?: Stack[];
  size?: 'small' | 'large';
}

export interface Visualization {
  title: string;
  description: string;
  type: string;
  url?: string;
  config?: string;
  inject?: boolean;
  stacks?: Stack[];
  size?: 'small' | 'large';
  columns?: RowColumn[];
}

export interface UIConfig {
  title: string;
  description: string;
  visualizations: Visualization[];
}

export interface Config {
  version: number;
  server: ServerConfig;
  monitor: MonitorConfig;
  css: CSSConfig;
  base_path: string;
  ui: UIConfig;
  config_d: Record<string, any>;
}

export type Status = 'blank' | 'red' | 'green' | 'yellow';

export interface MonitorStatus {
  status: Status;
  code: number;
  description: string;
  css: {
    metadata: Record<string, any>;
  };
  metadata: Record<string, any>;
  log: string[];
}

export interface MonitorConfigItem {
  interval: string;
  timeout: string;
  command: string;
}

export interface Monitor {
  id: string;
  config: MonitorConfigItem;
  status: MonitorStatus;
  children: Record<string, MonitorChildStatus>;
}

export interface MonitorChildStatus {
  axes: Record<string, any>;
  status: MonitorStatus;
}

export interface StatusData {
  config: Config;
  monitors: Monitor[];
}
