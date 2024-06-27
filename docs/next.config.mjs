import buildWithNextra from "nextra";
import { getHighlighter, BUNDLED_LANGUAGES } from "shiki";
import path from "path";
const withNextra = buildWithNextra({
	theme: "nextra-theme-docs",
	themeConfig: "./theme.config.tsx",
	latex: true,
	defaultShowCopyCode: false,
	readingTime: true,
	mdxOptions: {
		rehypePrettyCodeOptions: {
			getHighlighter: (options) =>
				getHighlighter({
					...options,
					langs: [
						...BUNDLED_LANGUAGES,
						// custom grammar options, see the Shiki documentation for how to provide these options
						{
							id: "rhai",
							scopeName: "source.rhai",
							aliases: ["rh"], // Along with id, aliases will be included in the allowed names you can use when writing markdown.
							// path: "./public/syntax/grammar.tmLanguage.json",
							path: path.resolve(
								process.cwd(),
								"./public/syntax/rhai.tmLanguage.json"
							),
						},
					],
				}),
		},
	},
});

export default withNextra({
	publicRuntimeConfig: {
		UMAMI_WEBSITE_ID: process.env.UMAMI_WEBSITE_ID,
	},
	output: "export",
	images: {
		unoptimized: true,
	},
});
