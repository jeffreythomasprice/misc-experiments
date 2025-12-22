import path from "path";
import { exec } from "./processes";

export async function getPdfPageCount({
	path,
}: {
	path: string;
}): Promise<number> {
	const processResult = await exec({
		command: "pdftk",
		arguments: [
			path,
			"dump_data",
		],
	});
	const result = processResult.stdout
		.split("\n")
		.map(s => s.trim())
		.map(s => /^NumberOfPages:\s*([0-9]+)$/.exec(s))
		.filter(x => !!x)
		.map(x => x[1])
		.filter((x): x is string => !!x)[0];
	if (!result) {
		throw new Error("didn't find number of pages result");
	}
	return parseInt(result, 10);
}

export async function extractPdfPagesIntoNewPdf({
	inputPath,
	outputDir,
	firstPage,
	lastPage,
}: {
	inputPath: string;
	outputDir: string;
	firstPage: number;
	lastPage: number;
}): Promise<string> {
	const inputFileName = path.basename(inputPath);
	const inputFileExtension = path.extname(inputFileName);
	const outputFileName = `${inputFileName.substring(0, inputFileName.length - inputFileExtension.length)}-${firstPage}-${lastPage}${inputFileExtension}`;
	const outputFilePath = path.join(outputDir, outputFileName);
	await exec({
		command: "pdftk",
		arguments: [
			inputPath,
			"cat",
			`${firstPage}-${lastPage}`,
			"output",
			outputFilePath,
		],
	});
	console.log(`wrote page range ${firstPage}-${lastPage} to ${outputFilePath}`);
	return outputFilePath;
}

export async function extractPdfText({
	path,
}: {
	path: string;
}): Promise<string> {
	const processResult = await exec({
		command: "pdftotext",
		arguments: [
			path,
			"-",
		],
	});
	return processResult.stdout;
}