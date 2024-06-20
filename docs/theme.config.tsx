import { DocsThemeConfig } from "nextra-theme-docs";
import React from "react";
import { useRouter } from "next/router";
import { useConfig } from "nextra-theme-docs";

const config: DocsThemeConfig = {
	project: {
		link: "https://github.com/cestef/rwalk",
	},
	faviconGlyph: "🔍",
	logo: (
		<>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 36 36"
				width="24"
				height="24"
				style={{ marginBottom: "-0.2em" }}
			>
				<path fill="#9AAAB4" d="M13.503 19.693l2.828 2.828-4.95 4.95-2.828-2.829z" />
				<path
					fill="#66757F"
					d="M1.257 29.11l5.88-5.879c.781-.781 2.047-.781 2.828 0l2.828 2.828c.781.781.781 2.047 0 2.828l-5.879 5.879c-1.562 1.563-4.096 1.563-5.658 0-1.561-1.561-1.56-4.094.001-5.656z"
				/>
				<circle fill="#8899A6" cx="22.355" cy="13.669" r="13.5" />
				<circle fill="#BBDDF5" cx="22.355" cy="13.669" r="9.5" />
			</svg>
			<span style={{ marginLeft: ".4em", fontWeight: 800, fontSize: 24 }}>rwalk</span>
		</>
	),
	docsRepositoryBase: "https://github.com/cestef/rwalk/tree/main/docs",
	primaryHue: 13,
	primarySaturation: 40,
	chat: {
		link: "https://cstef.dev/discord",
	},
	navigation: true,
	useNextSeoProps() {
		const { asPath } = useRouter();
		if (asPath !== "/") {
			return {
				titleTemplate: "%s – rwalk",
			};
		} else {
			return {
				title: "rwalk's Documentation",
			};
		}
	},
	head: () => {
		const { asPath, defaultLocale, locale } = useRouter();
		const { frontMatter } = useConfig();
		const url =
			"https://rwalk.cstef.dev" + (defaultLocale === locale ? asPath : `/${locale}${asPath}`);

		return (
			<>
				<meta property="og:url" content={url} />
				<meta property="og:title" content={frontMatter.title || "rwalk's Documentation"} />
				<meta
					property="og:description"
					content={
						frontMatter.description ||
						"A fast, lightweight and flexible web fuzzing tool"
					}
				/>
			</>
		);
	},
};

export default config;
