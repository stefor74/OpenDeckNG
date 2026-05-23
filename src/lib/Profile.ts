import type { ActionInstance } from "./ActionInstance.ts";

export type BackgroundMode = "stretch" | "tile" | "center" | "cover";

export type Profile = {
	device: string;
	id: string;
	keys: (ActionInstance | null)[];
	sliders: (ActionInstance | null)[];
	background_image?: string;
	background_color?: string;
	background_mode?: BackgroundMode;
};
