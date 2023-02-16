import { Logger } from "./utils";

Logger.defaultLevel = Logger.Level.Debug;
const logger = new Logger({
	prefix: "worker",
});

logger.debug("TODO worker");

addEventListener("message", (e) => {
	logger.debug("message", e.data);
	postMessage("this came from worker");
});
addEventListener("messageerror", (e) => {
	logger.error("messageerror", e);
});
addEventListener("error", (e) => {
	logger.error("error", e);
});
