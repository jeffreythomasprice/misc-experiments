import { useEffect, useState } from "react";

export interface ChannelSelectorProps {
	all: string[];
	current: string[];
	onChange: (active: string | null) => void;
}

export function ChannelSelector({ all, current, onChange }: ChannelSelectorProps) {
	let [active, setActive] = useState<string | number | string[] | undefined>(undefined);

	const doUpdate = (newValue: string | number | string[] | undefined) => {
		if (typeof newValue !== "string") {
			newValue = undefined;
		}
		active = newValue;
		setActive(active);
		onChange(active ?? null);
	};

	let [combined, setCombined] = useState<string[]>([]);
	useEffect(
		() => {
			combined = [...new Set([...all, ...current])];
			combined.sort((a, b) => a.localeCompare(b));
			setCombined(combined);

			if (typeof active === "string" && !combined.includes(active)) {
				if (combined.length === 0) {
					doUpdate(undefined);
				} else {
					doUpdate(combined[0]);
				}
			} else if (!active && combined.length > 0) {
				doUpdate(combined[0]);
			}
		},
		[all, current]
	);

	const options: JSX.Element[] = [];
	if (combined.length === 0) {
		options.push(<option key={"none"} value={undefined}>-- No Channels --</option>);
	} else {
		options.push(...combined.map((channel) => {
			return <option key={channel} value={channel}>{channel}</option>;
		}));
	}

	return <select value={active} onChange={(e) => {
		doUpdate(e.target.value);
	}}>
		{options}
	</select>;
}
