import { spawn } from "child_process";

export async function exec({
	command,
	arguments: args,
}: {
	command: string;
	arguments: string[];
}): Promise<{
	exitCode: number;
	stdout: string;
	stderr: string;
}> {
	const process = spawn(command, args);
	return new Promise((resolve, reject) => {
		let stdout = "";
		let stderr = "";

		process.stdout.on("data", (data) => {
			stdout += data.toString("utf-8");
		});

		process.stderr.on("data", (data) => {
			stderr += data.toString("utf-8");
		});

		process.on("close", (code, signal) => {
			if (code === 0) {
				resolve({
					exitCode: code,
					stdout,
					stderr,
				});
			} else if (typeof code === "number") {
				console.error(`${command} failed with exit code: ${code}\nstderr:\n${stderr.trim()}`);
				reject(new Error(`${command} exited with exit code: ${code}`));
			} else if (signal) {
				reject(new Error(`${command} exited with signal ${signal}`));
			} else {
				reject(new Error(`${command} exited with no exit code and no signal`));
			}
		});
	});
}