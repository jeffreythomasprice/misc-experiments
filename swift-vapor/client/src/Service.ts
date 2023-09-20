import { LoginRequest, LoginResponse } from "./models";

export class Service {
	constructor(private readonly baseUrl: string) { }

	async login(name: string): Promise<LoginResponse> {
		const requestBody: LoginRequest = {
			name,
		};
		const response = await fetch(
			`${this.baseUrl}/login`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(requestBody),
			}
		);
		assertOK(response);
		const responseBody = await response.json() as LoginResponse;
		return responseBody;
	}
}

function assertOK(response: Response) {
	if (!response.ok) {
		throw new Error(`http status code ${response.status}`);
	}
}