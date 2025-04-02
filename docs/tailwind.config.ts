import type { Config } from "tailwindcss";
import typography from "@tailwindcss/typography";

const hexToRgb = (h: string) => {
	let hex = h;
	hex = hex.replace("#", "");
	hex = hex.length === 3 ? hex.replace(/./g, "$&$&") : hex;
	const r = Number.parseInt(hex.substring(0, 2), 16);
	const g = Number.parseInt(hex.substring(2, 4), 16);
	const b = Number.parseInt(hex.substring(4, 6), 16);
	return `${r} ${g} ${b}`;
};

export default {
	darkMode: "class",
	content: [
		"./templates/**/*.html",
		"./templates/*.html",
		"./styles/**/*.css",
		"./styles/*.css",
	],

	plugins: [typography],
} satisfies Config;
