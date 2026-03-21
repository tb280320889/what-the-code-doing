export type UserId = string;

export interface AppConfig {
  port: number;
  host: string;
}

export enum Status {
  Active = "active",
  Inactive = "inactive",
}
