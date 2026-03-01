import pino, { type Logger } from "pino";

export function createLogger(logFilePath: string): Logger {
  return pino(
    {
      level: process.env.FAE_LOG_LEVEL ?? "info",
      base: undefined,
      timestamp: pino.stdTimeFunctions.isoTime
    },
    pino.destination({ dest: logFilePath, mkdir: true, sync: false })
  );
}
