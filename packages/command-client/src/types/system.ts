export type SystemHealth = {
  appName: string;
  storageReady: boolean;
  commandBusReady: boolean;
  aiReservedCapabilities: string[];
  syncStrategy: string;
};

export type DiagnosticBundleResult = {
  bundleDir: string;
  manifestPath: string;
};

export type AppEdition = "community" | "pro";

export type AppDistribution = "github" | "website";

export type AppChannel = "test" | "stable";

export type AppInfo = {
  name: string;
  version: string;
  edition: AppEdition;
  distribution: AppDistribution;
  channel: AppChannel;
  commit: string | null;
};
