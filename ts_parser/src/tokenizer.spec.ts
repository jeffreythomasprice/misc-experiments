import { filter } from "rxjs";

import { Tokenizer } from "./tokenizer";

enum TokenName {
	Number = "NUMBER",
	Whitespace = "WHITESPACE",
	Plus = "PLUS",
	Minus = "MINUS",
	Times = "TIMES",
	Divide = "DIVIDE",
	LeftParenthesis = "LEFT_PARENTHESIS",
	RightParenthesis = "RIGHT_PARENTHESIS",
}

describe("tokenizer", () => {
	it("TODO JEFF testing", async () => {
		const tokenizer = new Tokenizer.Builder()
			.add(TokenName.Number, /[0-9]+/)
			.add(TokenName.Whitespace, /\s+/)
			.add(TokenName.Plus, "+")
			.add(TokenName.Minus, "-")
			.add(TokenName.Times, "*")
			.add(TokenName.Divide, "/")
			.add(TokenName.LeftParenthesis, "(")
			.add(TokenName.RightParenthesis, ")")
			.build();
		await tokenizer.tokenize("(1 + 2)/3*-4")
			.pipe(filter((token) => token.name !== TokenName.Whitespace))
			.forEach((token) => {
				console.log(`TODO JEFF token observer ${token.name}, ${token.value}`);
			});
	});
});
