const releaseChannels = new Set(["test", "stable"]);

export function resolveReleaseChannel(explicitChannel) {
  if (explicitChannel === undefined || explicitChannel === "") return "test";
  if (!releaseChannels.has(explicitChannel)) {
    throw new Error(
      `UNFOUR_RELEASE_CHANNEL must be exactly "test" or "stable", got ${JSON.stringify(explicitChannel)}`,
    );
  }
  return explicitChannel;
}

export function tauriEnvironment(environment = process.env) {
  const channel = resolveReleaseChannel(environment.UNFOUR_RELEASE_CHANNEL);
  return {
    channel,
    environment: {
      ...environment,
      UNFOUR_RELEASE_CHANNEL: channel,
    },
  };
}
