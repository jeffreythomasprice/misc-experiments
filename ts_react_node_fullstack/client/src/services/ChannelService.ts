import { APIClient } from "../../../shared";
import { Observable, Subject } from "rxjs";

export class ChannelService {
	private readonly allChannels = new Subject<string[]>();
	private readonly currentChannels = new Subject<string[]>();

	constructor(private readonly apiClient: APIClient) {
		apiClient.authTokenObservable.subscribe(() => {
			void this.refresh();
		});
		void this.refresh();
	}

	get allChannelsObservable(): Observable<string[]> {
		return this.allChannels;
	}

	get currentChannelsObservable(): Observable<string[]> {
		return this.currentChannels;
	}

	async join(channel: string) {
		await this.apiClient.joinChannel(channel);
		await this.refresh();
	}

	async leave(channel: string) {
		await this.apiClient.leaveChannel(channel);
		await this.refresh();
	}

	private async refresh() {
		try {
			if (this.apiClient.needsLogin) {
				this.allChannels.next([]);
				this.currentChannels.next([]);
			} else {
				this.allChannels.next((await this.apiClient.getAllChannels()).channels);
				this.currentChannels.next((await this.apiClient.getCurrentChannels()).channels);
			}
		} catch (e) {
			console.error("error refreshing channels", e);
		}
	}
}
