import type { Config } from "tailwindcss";

export default {
	content: [
		"./pages/**/*.{js,jsx,ts,tsx,md,mdx}",
		"./components/**/*.{js,jsx,ts,tsx,md,mdx}",
	],
	theme: {
		extend: {},
	},
	plugins: [],
	darkMode: "selector",
} satisfies Config;
