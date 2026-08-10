const releaseChannels = new Set(["test", "stable"]);

export function resolveReleaseChannel(explicitChannel, defaultChannel = "test") {
  if (explicitChannel === undefined || explicitChannel === "") {
    return defaultChannel;
  }
  if (!releaseChannels.has(explicitChannel)) {
    throw new Error(
      `UNFOUR_RELEASE_CHANNEL must be exactly "test" or "stable", got ${JSON.stringify(explicitChannel)}`,
    );
  }
  return explicitChannel;
}

export function resolveTauriInvocation(args) {
  if (args[0] === "build:test") {
    return {
      args: ["build", ...args.slice(1)],
      defaultChannel: "test",
      forcedChannel: "test",
    };
  }

  return {
    args,
    defaultChannel: args[0] === "build" ? "stable" : "test",
    forcedChannel: undefined,
  };
}

export function tauriEnvironment(
  environment = process.env,
  defaultChannel = "test",
  forcedChannel,
) {
  const channel = resolveReleaseChannel(
    forcedChannel ?? environment.UNFOUR_RELEASE_CHANNEL,
    defaultChannel,
  );
  return {
    channel,
    environment: {
      ...environment,
      UNFOUR_RELEASE_CHANNEL: channel,
    },
  };
}
